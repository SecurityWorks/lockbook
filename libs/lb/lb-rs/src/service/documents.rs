use std::collections::HashSet;
use std::sync::atomic::Ordering;

use crate::logic::crypto::DecryptedDocument;
use crate::logic::file_like::FileLike;
use crate::logic::lazy::LazyTree;
use crate::logic::signed_file::SignedFile;
use crate::logic::tree_like::TreeLike;
use crate::logic::validate;
use crate::model::clock::get_time;
use crate::model::errors::{LbErrKind, LbResult};
use crate::model::file_metadata::{DocumentHmac, FileType};
use crate::Lb;
use uuid::Uuid;

use super::activity;

impl Lb {
    #[instrument(level = "debug", skip(self), err(Debug))]
    pub async fn read_document(&self, id: Uuid) -> LbResult<DecryptedDocument> {
        let tx = self.ro_tx().await;
        let db = tx.db();

        let mut tree = (&db.base_metadata).to_staged(&db.local_metadata).to_lazy();

        let doc = self.read_document_helper(id, &mut tree).await?;

        let bg_lb = self.clone();
        tokio::spawn(async move {
            bg_lb
                .add_doc_event(activity::DocEvent::Read(id, get_time().0))
                .await
                .unwrap();
        });

        Ok(doc)
    }

    #[instrument(level = "debug", skip(self, content), err(Debug))]
    pub async fn write_document(&self, id: Uuid, content: &[u8]) -> LbResult<()> {
        debug!("doc length: {}", content.len());
        let mut tx = self.begin_tx().await;
        let db = tx.db();

        let mut tree = (&db.base_metadata)
            .to_staged(&mut db.local_metadata)
            .to_lazy();
        let account = db.account.get().ok_or(LbErrKind::AccountNonexistent)?;

        let id = match tree.find(&id)?.file_type() {
            FileType::Document | FileType::Folder => id,
            FileType::Link { target } => target,
        };
        let encrypted_document = tree.update_document(&id, content, account)?;
        let hmac = tree.find(&id)?.document_hmac().copied();
        self.docs.insert(id, hmac, &encrypted_document).await?;
        tx.end();

        let bg_lb = self.clone();
        tokio::spawn(async move {
            bg_lb
                .add_doc_event(activity::DocEvent::Write(id, get_time().0))
                .await
                .unwrap();
            bg_lb.cleanup().await.unwrap();
        });

        self.spawn_build_index();

        Ok(())
    }

    #[instrument(level = "debug", skip(self), err(Debug))]
    pub async fn read_document_with_hmac(
        &self, id: Uuid,
    ) -> LbResult<(Option<DocumentHmac>, DecryptedDocument)> {
        let tx = self.ro_tx().await;
        let db = tx.db();

        let mut tree = (&db.base_metadata).to_staged(&db.local_metadata).to_lazy();

        let doc = self.read_document_helper(id, &mut tree).await?;
        let hmac = tree.find(&id)?.document_hmac().copied();

        let bg_lb = self.clone();
        tokio::spawn(async move {
            bg_lb
                .add_doc_event(activity::DocEvent::Read(id, get_time().0))
                .await
                .unwrap();
        });

        Ok((hmac, doc))
    }

    #[instrument(level = "debug", skip(self, content), err(Debug))]
    pub async fn safe_write(
        &self, id: Uuid, old_hmac: Option<DocumentHmac>, content: Vec<u8>,
    ) -> LbResult<DocumentHmac> {
        debug!("doc length: {}", content.len());
        let mut tx = self.begin_tx().await;
        let db = tx.db();

        let mut tree = (&db.base_metadata)
            .to_staged(&mut db.local_metadata)
            .to_lazy();

        let account = self.get_account()?;
        let file = tree.find(&id)?;
        if file.document_hmac() != old_hmac.as_ref() {
            return Err(LbErrKind::ReReadRequired.into());
        }
        let id = match file.file_type() {
            FileType::Document | FileType::Folder => id,
            FileType::Link { target } => target,
        };
        // todo can we not borrow here?
        let encrypted_document = tree.update_document(&id, &content, account)?;
        let hmac = tree.find(&id)?.document_hmac();
        let hmac = *hmac.ok_or_else(|| {
            LbErrKind::Unexpected(format!("hmac missing for a document we just wrote {}", id))
        })?;
        self.docs
            .insert(id, Some(hmac), &encrypted_document)
            .await?;
        tx.end();

        let bg_lb = self.clone();
        tokio::spawn(async move {
            bg_lb
                .add_doc_event(activity::DocEvent::Write(id, get_time().0))
                .await
                .unwrap();
            bg_lb.cleanup().await.unwrap();
        });

        self.spawn_build_index();

        Ok(hmac)
    }

    pub(crate) async fn cleanup(&self) -> LbResult<()> {
        // there is a risk that dont_delete is set to true after we check it
        if self.docs.dont_delete.load(Ordering::SeqCst) {
            debug!("skipping doc cleanup due to active sync");
            return Ok(());
        }

        let tx = self.ro_tx().await;
        let db = tx.db();

        let tree = db.base_metadata.stage(&db.local_metadata);

        let base_files = tree.base.all_files()?.into_iter();
        let local_files = tree.staged.all_files()?.into_iter();

        let file_hmacs = base_files
            .chain(local_files)
            .filter_map(|f| f.document_hmac().map(|hmac| (*f.id(), *hmac)))
            .collect::<HashSet<_>>();

        drop(tx);

        self.docs.retain(file_hmacs).await?;

        Ok(())
    }

    /// This fn is what will fetch the document remotely if it's not present locally
    pub(crate) async fn read_document_helper<T>(
        &self, id: Uuid, tree: &mut LazyTree<T>,
    ) -> LbResult<DecryptedDocument>
    where
        T: TreeLike<F = SignedFile>,
    {
        let file = tree.find(&id)?;
        validate::is_document(file)?;
        let hmac = file.document_hmac().copied();

        if tree.calculate_deleted(&id)? {
            return Err(LbErrKind::FileNonexistent.into());
        }

        let doc = match hmac {
            Some(hmac) => {
                let doc = self.docs.get(id, Some(hmac)).await?;
                tree.decrypt_document(&id, &doc, self.get_account()?)?
            }
            None => vec![],
        };

        Ok(doc)
    }
}