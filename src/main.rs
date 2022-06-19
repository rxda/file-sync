use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{self, Read};
use std::sync::mpsc::channel;
// use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = vec![SyncConfig {
        file_paths: [
            "/home/ubuntu/mc/data/world/playerdata/00000000-0000-0000-0009-00000b299750.dat".to_string(),
            "/home/ubuntu/mc/data/world/playerdata/ce7aa529-26dc-3d21-846c-1f604658c3d3.dat".to_string(),
        ],
        direction_type: DirectionType::Double,
    },SyncConfig{
        file_paths: [
            "/home/ubuntu/mc/data/world/playerdata/00000000-0000-0000-0009-00000b299750.dat_old".to_string(),
            "/home/ubuntu/mc/data/world/playerdata/ce7aa529-26dc-3d21-846c-1f604658c3d3.dat_old".to_string(),
        ],
        direction_type: DirectionType::Double,
    }];

    config_watch(&config).await.unwrap();

    loop {
        thread::sleep(std::time::Duration::from_secs(1000));
    }
}

enum DirectionType {
    Sigle,
    Double,
}

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
                    watch_file(path0.clone(), path1.clone()).await;
                });
            }
            DirectionType::Double => {
                let p0 = path0.clone();
                let p1 = path1.clone();
                tokio::spawn(async move {
                    watch_file(path0.clone(), path1.clone()).await;
                });
                tokio::spawn(async move {
                    watch_file(p1, p0).await;
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
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
    watcher.watch(&src, RecursiveMode::Recursive).unwrap();
    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Create(_path) => {
                        copy_file(&src, &dst).unwrap();
                    }
                    DebouncedEvent::Write(_path) => {
                        copy_file(&src, &dst).unwrap();
                    }
                    _ => (),
                };
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}

// async fn copy_file(src: &str, dst: &str) -> anyhow::Result<()> {
//     let mut src = File::open(src).await?;
//     let mut dst = File::create(dst).await?;

//     let mut src_contents = vec![];
//     src.read_buf(&mut src_contents).await?;
//     let mut dst_contents = vec![];
//     dst.read_buf(&mut dst_contents).await?;

//     let src_digest = md5::compute(src_contents);
//     let dst_digest = md5::compute(dst_contents);

//     if src_digest == dst_digest {
//         return Ok(());
//     }else{
//         io::copy(&mut src, &mut dst).await?;
//     }

//     Ok(())
// }

fn copy_file(src: &str, dst: &str) -> anyhow::Result<()> {
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
