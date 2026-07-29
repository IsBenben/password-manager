use git2::{Repository, Signature};
use std::path::Path;

pub struct GitSync;

impl GitSync {
    pub fn init(path: &Path) -> Result<(), String> {
        if Repository::open(path).is_ok() {
            return Ok(());
        }
        Repository::init(path).map_err(|e| format!("Git init error: {}", e))?;
        let mut config_path = path.to_path_buf();
        config_path.push(".git");
        let repo = Repository::open(path).map_err(|e| format!("Git open error: {}", e))?;
        let mut config = repo.config().map_err(|e| format!("Config error: {}", e))?;
        config
            .set_str("user.name", "Password Manager")
            .ok();
        config
            .set_str("user.email", "pm@local")
            .ok();
        Ok(())
    }

    pub fn commit_and_push(
        path: &Path,
        remote_url: &str,
        message: Option<&str>,
    ) -> Result<String, String> {
        Self::init(path)?;
        let repo = Repository::open(path).map_err(|e| format!("Open error: {}", e))?;

        let mut index = repo.index().map_err(|e| format!("Index error: {}", e))?;
        index
            .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Add error: {}", e))?;
        index
            .write_tree()
            .map_err(|e| format!("Tree write error: {}", e))?;

        let oid = index
            .write_tree()
            .map_err(|e| format!("Tree write error: {}", e))?;
        let tree = repo
            .find_tree(oid)
            .map_err(|e| format!("Tree find error: {}", e))?;

        let signature = Signature::now("Password Manager", "pm@local")
            .map_err(|e| format!("Signature error: {}", e))?;
        let msg = message.unwrap_or("Update password data");
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            msg,
            &tree,
            parent.as_ref().map(|c| vec![c]).unwrap_or_default().as_slice(),
        )
        .map_err(|e| format!("Commit error: {}", e))?;

        if !remote_url.is_empty() {
            let mut remote = repo
                .find_remote("origin")
                .or_else(|_| repo.remote("origin", remote_url))
                .map_err(|e| format!("Remote error: {}", e))?;

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key_from_agent(
                    username_from_url.unwrap_or("git"),
                )
            });

            let mut push_opts = git2::PushOptions::new();
            push_opts.remote_callbacks(callbacks);

            remote
                .push(&["refs/heads/master"], Some(&mut push_opts))
                .or_else(|_| {
                    let mut callbacks = git2::RemoteCallbacks::new();
                    callbacks.credentials(|_url, _username, _allowed_types| {
                        git2::Cred::default()
                    });
                    let mut push_opts = git2::PushOptions::new();
                    push_opts.remote_callbacks(callbacks);
                    remote.push(&["refs/heads/master"], Some(&mut push_opts))
                })
                .map_err(|e| format!("Push error: {}", e))?;
        }

        Ok("Committed and pushed successfully".to_string())
    }

    pub fn pull(path: &Path, remote_url: &str) -> Result<String, String> {
        if path.join(".git").exists() {
            let repo = Repository::open(path).map_err(|e| format!("Open error: {}", e))?;

            if !remote_url.is_empty() {
                repo.find_remote("origin").or_else(|_| {
                    repo.remote("origin", remote_url)
                        .map_err(|e| format!("Remote create error: {}", e))
                })?;

                let mut remote = repo
                    .find_remote("origin")
                    .map_err(|_| "Remote not found".to_string())?;

                let mut callbacks = git2::RemoteCallbacks::new();
                callbacks.credentials(|_url, _username, _allowed_types| git2::Cred::default());

                let mut fetch_opts = git2::FetchOptions::new();
                fetch_opts.remote_callbacks(callbacks);

                remote
                    .fetch(&["refs/heads/master"], Some(&mut fetch_opts), None)
                    .map_err(|e| format!("Fetch error: {}", e))?;

                let fetch_head = repo
                    .find_reference("FETCH_HEAD")
                    .map_err(|_| "FETCH_HEAD not found".to_string())?;
                let fetch_commit = repo
                    .reference_to_annotated_commit(&fetch_head)
                    .map_err(|_| "Not an annotation".to_string())?;

                let analysis = repo
                    .merge_analysis(&[&fetch_commit])
                    .map_err(|e| format!("Merge analysis error: {}", e))?;

                if analysis.0.is_up_to_date() {
                    return Ok("Already up to date".to_string());
                }

                if analysis.0.is_fast_forward() {
                    let mut reference = repo
                        .find_reference("refs/heads/master")
                        .map_err(|e| format!("Reference error: {}", e))?;
                    let fetch_oid = fetch_commit.id();
                    reference
                        .set_target(fetch_oid, "Fast-forward")
                        .map_err(|e| format!("Set target error: {}", e))?;
                    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                        .map_err(|e| format!("Checkout error: {}", e))?;
                    Ok("Pulled and fast-forwarded".to_string())
                } else {
                    Err("Merge required but not supported; local changes preserved".to_string())
                }
            } else {
                Err("No remote configured".to_string())
            }
        } else {
            Err("Not a git repository".to_string())
        }
    }
}
