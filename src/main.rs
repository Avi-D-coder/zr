use std::{env,fmt};
use std::path::PathBuf;

const VERSION: &'static str = "0.0.1";

#[derive(Debug)]
struct Plugin {
    repo: PathBuf,
    files: Vec<PathBuf>
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = self.repo.display();
        for file in &self.files {
            let basedir = file.parent().unwrap().display();
            write!(f, r"source {}/{}", prefix, file.display())?;
            write!(f, r"fpath+={}/{}/", prefix, basedir)?;
            write!(f, r"PATH=={}/{}:$PATH", prefix, basedir)?;
        }
        write!(f, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
    }
}

impl Plugin {
    pub fn from_path(path: PathBuf) -> Plugin {
        let path_clone = path.clone();
        let name = path_clone.iter().last().unwrap();
        let files: Vec<_> = path.read_dir().unwrap().filter_map(std::result::Result::ok).map(|file| file.path()).filter(|file| file.is_file()).collect();

        let antigen_plugin_file = files.iter().find(|file| file.file_name().unwrap() == path.join(name).join(".plugin.zsh"));
        if antigen_plugin_file.is_some() {
            return Plugin {
                repo: path,
                files: vec![antigen_plugin_file.unwrap().to_owned()]
            }
        }

        // prezto: if we find init.zsh, try to load with pmodload, or manually
        let prezto_plugin_file = files.iter().find(|file| file.file_name().unwrap() == path.join("init.zsh"));
        if prezto_plugin_file.is_some() {
            return match std::process::Command::new("pmodload").arg(name.clone()).spawn() {
                Ok(_) =>
                    Plugin {
                        repo: path,
                        files: vec![]
                    },
                Err(_) =>
                    Plugin {
                        repo: path,
                        files: vec![prezto_plugin_file.unwrap().to_owned()]
                    }
            }
        }

        // zsh plugins
        let zsh_plugin_files: Vec<_> = files.iter().filter(|file| file.extension().unwrap() == "zsh").map(|e| e.to_owned()).collect();
        if ! zsh_plugin_files.is_empty() {
            return Plugin {
                repo: path,
                files: zsh_plugin_files
            }
        }

        // sh plugins
        let sh_plugin_files: Vec<_> = files.iter().filter(|file| file.extension().unwrap() == "sh").map(|e| e.to_owned()).collect();
        if ! sh_plugin_files.is_empty() {
            return Plugin {
                repo: path,
                files: sh_plugin_files.to_vec()
            }
        }

        Plugin { repo: path, files: vec![] }
    }
}

fn main() {
    let default_home = format!("{}/.zr", env!("HOME"));
    let zr_home = PathBuf::from(option_env!("ZR_HOME").unwrap_or(default_home.as_str()));

    match env::args().nth(1) {
        Some(command) => {
            match command.as_ref() {
                "version" => version(),
                "debug" => debug(zr_home),
                "load" => load(zr_home, PathBuf::from(env::args().nth(2).unwrap())),
                _ => help(),
            }
        },
        None => help()
    };
}

fn debug(zr_home: PathBuf) {
    version();
    println!("  ZR_HOME: {}", zr_home.display());
}

fn help() {
    println!(r"zr {}

usage:
  zr [<plugin>|command]


commands:
  zr load <plugin> - save 'plugin' to ~/.zr-init.zsh
  zr help - print this help
  zr version - print the version
  zr debug - print environment vars",
      VERSION);
}

fn version() {
    println!("{}", VERSION);
}

fn load(zr_home: PathBuf, name: PathBuf) {
    println!("loading {:?}", name.display());
    let plugin_path = format!("{}/plugins/{}", zr_home.display(), name.display());
    println!("from path {:?}", plugin_path);
    let plugin = Plugin::from_path(PathBuf::from(&plugin_path));
    println!("loaded {:?}", name.display());
    println!("{}", plugin);
    println!("{:?}", plugin);
}
