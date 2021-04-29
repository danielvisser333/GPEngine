use winit::{event_loop::EventLoop, window::Window};
use log::error;

mod renderer;

const MAX_LOG_COUNT : u16= 14;

fn main(){
    logging::create_logger();
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap_or_else(|e|{error!("Failed to create window {}.",e);panic!("Failed to create window")});
    let _renderer = renderer::Renderer::new(&window);
}
mod logging{
    use std::fs::{File, OpenOptions};

    use directories::ProjectDirs;
    use log::{LevelFilter,info};
    use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger, ThreadLogMode};

    use crate::MAX_LOG_COUNT;
    pub fn create_logger(){
        let config = ConfigBuilder::new().set_thread_mode(ThreadLogMode::Both).build();
        let mut log_level = LevelFilter::Info;
        for arg in std::env::args(){
            if arg == "--trace" {log_level = LevelFilter::Trace}
            if arg == "--debug" {log_level = LevelFilter::Debug}
            if arg == "--nolog" {log_level = LevelFilter::Off}
        }
        CombinedLogger::init(
            vec![
                create_term_logger(log_level, config.clone()),
                create_write_logger(log_level, config)
            ]
        ).expect("Failed to create logger!");
        info!("Created logger with log level : {}.",log_level);
    }
    fn create_term_logger(log_level : LevelFilter , config : Config) -> Box<TermLogger>{
        return TermLogger::new(log_level, config, TerminalMode::Mixed, ColorChoice::Auto);
    }
    fn create_write_logger(log_level : LevelFilter , config : Config) -> Box<WriteLogger<File>>{
        let project_dirs = ProjectDirs::from("com", "gpengine", "gpengine").unwrap();
        let log_dir = project_dirs.data_dir().join("log");
        let timestamp = chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let log_file = log_dir.join(format!("log-{}.log",timestamp));
        if !log_file.parent().unwrap().exists() {std::fs::create_dir_all(log_file.parent().unwrap()).expect("Failed to create log directory.")} else{
            //prevent the creation of too many logs.
            let mut logs = std::fs::read_dir(log_file.parent().unwrap()).unwrap_or_else(|_|{panic!("Failed to read log directory")}).map(|r|r.unwrap().path()).collect::<Vec<_>>();
            logs.sort_by_key(|file|{file.file_name().unwrap().to_str().unwrap().chars().filter(|v|v.is_numeric()).collect::<String>().parse::<u64>().unwrap()});
            if logs.len() as u16 > MAX_LOG_COUNT{
                for i in 0..logs.len()-MAX_LOG_COUNT as usize{
                    println!("Removing log file {:?}.",logs[i]);
                    std::fs::remove_file(logs[i].clone()).unwrap();
                }
            }
        }
        if !log_file.exists(){std::fs::File::create(log_file.clone()).expect("Failed to create log file");}
        let writer = OpenOptions::new().write(true).open(log_file).expect("Failed to write to log file");
        return WriteLogger::new(log_level, config, writer);
    }
}