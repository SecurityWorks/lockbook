use uuid::Uuid;

use lockbook_core::model::account::Account;
use lockbook_core::model::crypto::DecryptedValue;
use lockbook_core::model::file_metadata::FileMetadata;
use lockbook_core::model::state::Config;
use lockbook_core::model::work_unit::WorkUnit;
use lockbook_core::service::db_state_service::State as DbState;
use lockbook_core::service::sync_service::WorkCalculated;
use lockbook_core::{
    calculate_work, create_account, create_file_at_path, execute_work, export_account, get_account,
    get_children, get_db_state, get_file_by_id, get_file_by_path, get_last_synced, get_root,
    import_account, list_paths, migrate_db, read_document, set_last_synced, write_document,
    AccountExportError, CalculateWorkError, CreateAccountError, Error as CoreError,
    ExecuteWorkError, GetAccountError, SetLastSyncedError,
};

fn api_url() -> String {
    match std::env::var("LOCKBOOK_API_URL") {
        Ok(path) => path,
        Err(_) => "http://qa.lockbook.app:8000".to_string(),
    }
}

pub enum LbSyncMsg {
    Doing(String),
    Error(String),
    Done,
}

pub struct LbCore {
    config: Config,
}

impl LbCore {
    pub fn new(cfg_path: &str) -> Self {
        Self {
            config: Config {
                writeable_path: cfg_path.to_string(),
            },
        }
    }

    pub fn init_db(&self) -> Result<(), String> {
        match get_db_state(&self.config) {
            Ok(state) => match state {
                DbState::ReadyToUse => Ok(()),
                DbState::Empty => Ok(()),
                DbState::MigrationRequired => {
                    println!("Local state requires migration! Performing migration now...");
                    match migrate_db(&self.config) {
                        Ok(_) => {
                            println!("Migration Successful!");
                            Ok(())
                        }
                        Err(err) => Err(format!("{:?}", err)),
                    }
                }
                DbState::StateRequiresClearing => Err(
                    "Your local state cannot be migrated, please re-sync with a fresh client."
                        .to_string(),
                ),
            },
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn create_account(&self, username: &str) -> Result<(), String> {
        match create_account(&self.config, &username, &api_url()) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                CoreError::UiError(err) => match err {
                    CreateAccountError::UsernameTaken => Err(String::from("Username taken.")),
                    CreateAccountError::InvalidUsername => {
                        Err(format!("Username '{}' is not valid (a-z || 0-9)", username))
                    }
                    CreateAccountError::CouldNotReachServer => {
                        Err(String::from("Could not reach server."))
                    }
                    CreateAccountError::AccountExistsAlready => {
                        Err(String::from("An account already exists."))
                    }
                    CreateAccountError::ClientUpdateRequired => {
                        Err(String::from("Client upgrade required."))
                    }
                },
                CoreError::Unexpected(msg) => panic!(msg),
            },
        }
    }

    pub fn import_account(&self, privkey: &str) -> Result<(), String> {
        match import_account(&self.config, privkey) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("error importing: {:?}", err)),
        }
    }

    pub fn export_account(&self) -> Result<String, String> {
        match export_account(&self.config) {
            Ok(acct_str) => Ok(acct_str),
            Err(err) => match err {
                CoreError::UiError(AccountExportError::NoAccount) => {
                    Err("Unable to load account".to_string())
                }
                CoreError::Unexpected(msg) => Err(msg),
            },
        }
    }

    pub fn account(&self) -> Result<Option<Account>, String> {
        match get_account(&self.config) {
            Ok(acct) => Ok(Some(acct)),
            Err(err) => match err {
                CoreError::UiError(GetAccountError::NoAccount) => Ok(None),
                CoreError::Unexpected(err) => {
                    println!("error getting account: {}", err);
                    Err("Unable to load account.".to_string())
                }
            },
        }
    }

    pub fn root(&self) -> Result<FileMetadata, String> {
        match get_root(&self.config) {
            Ok(root) => Ok(root),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn children(&self, parent: &FileMetadata) -> Result<Vec<FileMetadata>, String> {
        match get_children(&self.config, parent.id) {
            Ok(files) => Ok(files),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn create_file_at_path(&self, path: &str) -> Result<FileMetadata, String> {
        let prefixed = format!("{}/{}", self.root().unwrap().name, path);
        match create_file_at_path(&self.config, &prefixed) {
            Ok(file) => Ok(file),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn file_by_id(&self, id: Uuid) -> Result<FileMetadata, String> {
        match get_file_by_id(&self.config, id) {
            Ok(file) => Ok(file),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn file_by_path(&self, path: &str) -> Result<FileMetadata, String> {
        let acct = self.account().unwrap().unwrap();
        let p = format!("{}/{}", acct.username, path);

        match get_file_by_path(&self.config, &p) {
            Ok(file) => Ok(file),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn list_paths(&self) -> Result<Vec<String>, String> {
        match list_paths(&self.config, None) {
            Ok(paths) => Ok(paths),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn full_path_for(&self, f: &FileMetadata) -> String {
        let root_id = match self.root() {
            Ok(root) => {
                if f.id == root.id {
                    return "/".to_string();
                }
                root.id
            }
            Err(_) => Default::default(),
        };

        let mut path = "".to_string();
        let mut ff = f.clone();
        while ff.id != root_id {
            path.insert_str(0, &format!("/{}", ff.name));
            ff = match self.file_by_id(ff.parent) {
                Ok(f) => f,
                Err(_) => break,
            }
        }

        path
    }

    pub fn save(&self, id: Uuid, content: String) -> Result<(), String> {
        let dec = DecryptedValue::from(content);

        match write_document(&self.config, id, &dec) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn open(&self, id: &Uuid) -> Result<(FileMetadata, String), String> {
        match self.file_by_id(*id) {
            Ok(meta) => match self.read(meta.id) {
                Ok(decrypted) => Ok((meta, decrypted.secret)),
                Err(err) => Err(err),
            },
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn read(&self, id: Uuid) -> Result<DecryptedValue, String> {
        match read_document(&self.config, id) {
            Ok(dval) => Ok(dval),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn sync(&self, chan: &glib::Sender<LbSyncMsg>) -> Result<(), String> {
        let account = self.account().unwrap().unwrap();

        let mut work_calculated = match self.calculate_work() {
            Ok(work) => work,
            Err(err) => return Err(format!("{:?}", err)),
        };

        while !work_calculated.work_units.is_empty() {
            for work_unit in work_calculated.work_units {
                let (prefix, meta) = match &work_unit {
                    WorkUnit::LocalChange { metadata } => ("Pushing", metadata),
                    WorkUnit::ServerChange { metadata } => ("Pulling", metadata),
                };

                let path = self.full_path_for(&meta);
                let action = format!("{}: {}", prefix, path);
                chan.send(LbSyncMsg::Doing(action.clone())).unwrap();

                if let Err(err) = self.do_work(&account, work_unit) {
                    chan.send(LbSyncMsg::Error(format!("{}: {:?}", action, err)))
                        .unwrap();
                }
            }

            work_calculated = match self.calculate_work() {
                Ok(work) => work,
                Err(err) => return Err(format!("{:?}", err)),
            };
        }

        match self.set_last_synced(work_calculated.most_recent_update_from_server) {
            Ok(_) => {
                chan.send(LbSyncMsg::Done).unwrap();
                Ok(())
            }
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    pub fn calculate_work(&self) -> Result<WorkCalculated, CoreError<CalculateWorkError>> {
        calculate_work(&self.config)
    }

    fn do_work(&self, a: &Account, wu: WorkUnit) -> Result<(), CoreError<ExecuteWorkError>> {
        execute_work(&self.config, &a, wu)
    }

    fn set_last_synced(&self, last_sync: u64) -> Result<(), CoreError<SetLastSyncedError>> {
        set_last_synced(&self.config, last_sync)
    }

    pub fn get_last_synced(&self) -> Result<u64, String> {
        match get_last_synced(&self.config) {
            Ok(last) => Ok(last),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}