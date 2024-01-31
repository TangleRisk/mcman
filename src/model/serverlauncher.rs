use serde::{Deserialize, Serialize};
use std::{borrow::ToOwned, collections::HashMap};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PresetFlags {
    Aikars,
    Proxy,
    #[default]
    None,
}

impl PresetFlags {
    pub fn get_flags(&self) -> Vec<String> {
        match self {
            Self::Aikars => include_str!("../../res/aikars_flags"),
            Self::Proxy => include_str!("../../res/proxy_flags"),
            Self::None => "",
        }
        .split(char::is_whitespace)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct ServerLauncher {
    pub eula_args: bool,

    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub nogui: bool,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub preset_flags: PresetFlags,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub disable: bool,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub jvm_args: String,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub game_args: String,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub memory: String,
    #[serde(skip_serializing_if = "crate::util::is_default")]
    pub properties: HashMap<String, String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prelaunch: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub postlaunch: Vec<String>,

    pub java_version: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StartupMethod {
    Jar(String),
    Custom {
        windows: Vec<String>,
        linux: Vec<String>,
    },
}

impl ServerLauncher {
    pub fn get_java(&self) -> String {
        if let Some(Some(path)) = self
            .java_version
            .as_ref()
            .map(|v| std::env::var(format!("JAVA_{v}_BIN")).ok())
        {
            path
        } else {
            std::env::var("JAVA_BIN").unwrap_or(String::from("java"))
        }
    }

    pub fn generate_script_linux(&self, _servername: &str, startup: &StartupMethod) -> String {
        format!(
            "#!/bin/sh\n# generated by mcman\n{} {} \"$@\"\n",
            self.get_java(),
            self.get_arguments(startup, "linux").join(" ")
        )
    }

    pub fn generate_script_win(&self, servername: &str, startup: &StartupMethod) -> String {
        format!(
            "@echo off\r\n:: generated by mcman\r\ntitle {servername}\r\n{} {} %*\r\n",
            self.get_java(),
            self.get_arguments(startup, "windows").join(" ")
        )
    }

    pub fn get_arguments(&self, startup: &StartupMethod, platform: &str) -> Vec<String> {
        let mut args = self
            .jvm_args
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        if std::env::var("MC_MEMORY").is_ok() || !self.memory.is_empty() {
            let m = std::env::var("MC_MEMORY").unwrap_or(self.memory.clone());
            args.extend([format!("-Xms{m}"), format!("-Xmx{m}")]);
        }

        args.append(&mut self.preset_flags.get_flags());

        if self.eula_args {
            args.push(String::from("-Dcom.mojang.eula.agree=true"));
        }

        for (key, value) in &self.properties {
            args.push(format!(
                "-D{}={}",
                key,
                if value.contains(char::is_whitespace) {
                    "\"".to_owned() + value + "\""
                } else {
                    value.clone()
                }
            ));
        }

        args.extend(match startup.clone() {
            StartupMethod::Jar(jar) => vec![String::from("-jar"), jar],
            StartupMethod::Custom { linux, windows } => match platform {
                "linux" => linux,
                "windows" => windows,
                _ => vec![],
            },
        });

        if self.nogui && !matches!(self.preset_flags, PresetFlags::Proxy) {
            args.push(String::from("--nogui"));
        }

        args.extend(self.game_args.split_whitespace().map(ToOwned::to_owned));

        args
    }
}

impl Default for ServerLauncher {
    fn default() -> Self {
        Self {
            preset_flags: PresetFlags::None,
            nogui: true,
            jvm_args: String::new(),
            game_args: String::new(),
            disable: false,
            eula_args: true,
            memory: String::new(),
            properties: HashMap::default(),
            prelaunch: vec![],
            postlaunch: vec![],
            java_version: None,
        }
    }
}
