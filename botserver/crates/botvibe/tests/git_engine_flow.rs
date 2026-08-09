use botvibe::harness::cmd::run;
use std::path::Path;

fn git(cwd: &Path, args: &[&str]) -> String {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let out = run("git", &args, cwd, 60)
        .unwrap_or_else(|e| panic!("git {} failed: {e}", args.join(" ")));
    assert_eq!(
        out.exit_code,
        Some(0),
        "git {} failed: {}",
        args.join(" "),
        out.stderr
    );
    out.stdout
}

#[test]
fn full_git_flow_clone_status_diff_commit_push() {
    let base = std::env::temp_dir().join(format!("gbtest-gitflow-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).unwrap();
    let remote = base.join("remote.git");
    let work = base.join("work");

    git(&base, &["init", "--bare", remote.to_str().unwrap()]);
    git(&base, &["clone", remote.to_str().unwrap(), work.to_str().unwrap()]);
    git(&work, &["config", "user.name", "Test Bot"]);
    git(&work, &["config", "user.email", "test@generalbots.com"]);

    let file = work.join("hello.txt");
    std::fs::write(&file, "v1\n").unwrap();

    let status = git(&work, &["status", "--porcelain"]);
    assert!(status.contains("hello.txt"), "status should show new file, got {status}");

    git(&work, &["add", "hello.txt"]);
    let diff = git(&work, &["diff", "--cached"]);
    assert!(diff.contains("v1"), "diff should contain new content");

    git(&work, &["commit", "-m", "add hello"]);
    git(&work, &["push", "origin", "HEAD"]);

    let rev = git(&remote, &["rev-parse", "HEAD"]);
    assert!(!rev.trim().is_empty(), "bare remote must contain the pushed commit");

    let local_head = git(&work, &["rev-parse", "HEAD"]);
    assert_eq!(rev.trim(), local_head.trim(), "remote HEAD should equal local HEAD");

    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn git_guard_rejects_unknown_commands() {
    let out = run("definitely-not-a-real-binary", &[], std::path::Path::new("/tmp"), 5);
    assert!(out.is_err(), "unknown commands must be rejected");
}
