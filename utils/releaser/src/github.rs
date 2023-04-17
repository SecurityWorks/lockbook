use crate::utils::{core_version, lb_repo};
use crate::{utils, Github};
use gh_release::release::{CreateReleaseInfo, TagInfo};
use gh_release::ReleaseClient;

pub fn create_gh_release(gh: &Github) {
    let client = ReleaseClient::new(gh.0.clone()).unwrap();

    let tag_info = TagInfo {
        tag: core_version(),
        message: "".to_string(),
        object: utils::commit_hash(),
        type_tagged: "commit".to_string(),
    };

    client.create_a_tag(&lb_repo(), &tag_info).unwrap();

    let release_info = CreateReleaseInfo {
        tag_name: core_version(),
        target_commitish: None,
        name: Some(core_version()),
        body: None,
        draft: None,
        prerelease: None,
        discussion_category_name: None,
        generate_release_notes: Some(true),
        make_latest: Some("true".to_string()),
    };

    client.create_a_release(&lb_repo(), &release_info).unwrap();
}