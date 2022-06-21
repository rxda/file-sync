use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{Read};
use std::sync::mpsc::channel;
use std::{thread, env};
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file_path = args.get(1).unwrap();
    let mut config_file = File::open(config_file_path).unwrap();
    let mut config_content = vec![];
    config_file.read_to_end(&mut config_content).unwrap();
    let config: Vec<SyncConfig> = serde_json::from_slice(&config_content).unwrap();


    config_watch(&config).await.unwrap();

    loop {
        thread::sleep(std::time::Duration::from_secs(1000));
    }
}

#[derive(Serialize, Deserialize)]
enum DirectionType {
    Sigle,
    Double,
}
#[derive(Serialize, Deserialize)]
struct SyncConfig {
    file_paths: [String; 2],
    direction_type: DirectionType,
}

async fn config_watch(configs: &Vec<SyncConfig>) -> anyhow::Result<()> {
    for config in configs {
        let path0 = config.file_paths[0].clone();
        let path1 = config.file_paths[1].clone();
        match config.direction_type {
            DirectionType::Sigle => {
                tokio::spawn(async move {
                    watch_file(path0.clone(), path1.clone()).await.unwrap();
                });
            }
            DirectionType::Double => {
                let p0 = path0.clone();
                let p1 = path1.clone();
                tokio::spawn(async move {
                    watch_file(path0.clone(), path1.clone()).await.unwrap();
                });
                tokio::spawn(async move {
                    watch_file(p1, p0).await.unwrap();
                });
            }
        }
    }
    Ok(())
}

async fn watch_file(src: String, dst: String) -> anyhow::Result<()> {
    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10))?;
    watcher.watch(&src, RecursiveMode::Recursive)?;
    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Create(_path) => {
                        copy_file_with_log(&src, &dst);
                    }
                    DebouncedEvent::Write(_path) => {
                        copy_file_with_log(&src, &dst);
                    }
                    _ => (),
                };
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
fn copy_file_with_log(src: &str, dst: &str){
    match copy_file(src,dst){
        Ok(_) => (),
        Err(e) => println!("copy {} to {} error: {:?}", src, dst, e),
    }
}


fn copy_file(src: &str, dst: &str) -> anyhow::Result<()> {
    let src_modify_time = fs::metadata(src)?.modified()?;
    let dst_modify_time = fs::metadata(dst)?.modified()?;
    
    if dst_modify_time.gt(&src_modify_time){
        return Ok(());
    }

    let mut src_file = File::open(src)?;
    let mut dst_file = File::open(dst)?;

    let mut src_contents = vec![];
    src_file.read_to_end(&mut src_contents)?;
    let mut dst_contents = vec![];
    dst_file.read_to_end(&mut dst_contents)?;

    let src_digest = md5::compute(&src_contents);
    let dst_digest = md5::compute(&dst_contents);

    if src_digest == dst_digest {
        return Ok(());
    } else {
        fs::write(dst, src_contents)?;
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_get_encrypt_password() {
//         watch_file(
//             "/Users/rxda/code/rust/mc-user-sync/test-file/a.txt".to_string(),
//             "/Users/rxda/code/rust/mc-user-sync/test-file/b.txt".to_string(),
//         );
//     }
// }
