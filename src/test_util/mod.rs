use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

pub fn temp_db_path() -> String {
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("could not get system time")
        .as_micros();

    let path = get_path_prefix();
    let p = format!("{}/{:?}", path, t);

    std::fs::create_dir_all(&*p).expect("could not create test directory");
    p
}

pub fn temp_manifest_path() -> PathBuf {
    let mut buf = PathBuf::from(temp_db_path());
    buf.push("manifest");
    buf
}

fn get_path_prefix() -> String {
    // for running on github actions
    match env::var("GITHUB_WORKSPACE") {
        Ok(val) => format!("{}/test_remits", val),
        Err(_) => "/tmp/test_remits".into(),
    }
}
