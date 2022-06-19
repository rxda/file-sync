use notify::{DebouncedEvent, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use same_file::is_same_file;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = vec![SyncConfig {
        file_paths: ["a.txt".to_string(), "b.txt".to_string()],
        direction_type: DirectionType::Double,
    }];

    config_watch(&config).await.unwrap();
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
                tokio::spawn(watch_file(&mut config.file_paths[0],&mut path1));
                // watch_file(&config.file_paths[0], &config.file_paths[1]).await?;
            }
            DirectionType::Double => {
                // watch_file(&config.file_paths[0], &config.file_paths[1]).await?;
                // watch_file(&config.file_paths[1], &config.file_paths[0]).await?;
            }
        }
    }
    Ok(())
}

async fn watch_file<'a>(src: &'a mut str, dst: &'a mut str) -> anyhow::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = tokio::sync::mpsc::channel::<DebouncedEvent>(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new( move|result: Result<Event, Error>| {
            let event = result.unwrap();

            println!("{:?}", event);
            match event {
                DebouncedEvent::Create(_path) => {
                    copy_file(src, dst).unwrap();
                }
                DebouncedEvent::Write(_path) => {
                    copy_file(src, dst).unwrap();
                }
                _ => (),
            };
        })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&Path::new(src), RecursiveMode::Recursive)?;
    Ok(())
}

fn copy_file(src: &str, dst: &str) -> anyhow::Result<()> {
    if !is_same_file(src, dst)? {
        let mut src = File::open(src)?;
        let mut dst = File::create(dst)?;
        io::copy(&mut src, &mut dst)?;
    }
    Ok(())
}
