#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::error::Error;
use std::str::FromStr;
use std::thread::{sleep, spawn};
use std::time::Duration;

use persy::{Config as PersyConfig, Persy, PersyId, SegmentId};
use serde::{Deserialize, Serialize};
use tauri::{Manager, SystemTray, SystemTrayEvent};
use wifi_rs::{prelude::*, WiFi as WifiClient};
use wifiscanner::{self, Wifi};

#[derive(Clone, serde::Serialize)]
struct Payload {
    wifi_list: Vec<WiFi>,
}

#[derive(Clone, serde::Serialize)]
struct WiFi {
    ssid: String,
    mac: String,
    security: String,
    signal_level: String,
    channel: String,
    connected: bool,
}
impl WiFi {
    fn new(wifi: &Wifi, connected: bool) -> Self {
        Self {
            ssid: wifi.ssid.clone(),
            mac: wifi.mac.clone(),
            security: wifi.security.clone(),
            signal_level: wifi.signal_level.to_string(),
            channel: wifi.channel.to_string(),
            connected,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct WifiInfo {
    agrCtlRSSI: String,
    agrExtRSSI: String,
    agrCtlNoise: String,
    agrExtNoise: String,
    state: String,
    #[serde(rename = "op mode")]
    op_mode: String,
    lastTxRate: String,
    maxRate: String,
    lastAssocStatus: String,
    #[serde(rename = "802.11 auth")]
    auth: String,
    #[serde(rename = "link_auth")]
    link_auth: String,
    BSSID: Option<String>,
    SSID: String,
    MCS: String,
    guardInterval: String,
    NSS: String,
    channel: Vec<String>,
}

impl FromStr for WifiInfo {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let agrCtlRSSI = lines.next().unwrap().trim().to_string();
        let agrExtRSSI = lines.next().unwrap().trim().to_string();
        let agrCtlNoise = lines.next().unwrap().trim().to_string();
        let agrExtNoise = lines.next().unwrap().trim().to_string();
        let state = lines.next().unwrap().trim().to_string();
        let op_mode = lines.next().unwrap().trim().to_string();
        let lastTxRate = lines.next().unwrap().trim().to_string();
        let maxRate = lines.next().unwrap().trim().to_string();
        let lastAssocStatus = lines.next().unwrap().trim().to_string();
        let auth = lines.next().unwrap().trim().to_string();
        let link_auth = lines.next().unwrap().trim().to_string();
        let BSSID = match lines.next() {
            Some(line) => match line.len() > 0 {
                true => Some(line.trim().to_string()),
                false => None,
            },
            None => None,
        };
        let SSID = lines.next().unwrap().trim().to_string();
        let MCS = lines.next().unwrap().trim().to_string();
        let guardInterval = lines.next().unwrap().trim().to_string();
        let NSS = lines.next().unwrap().trim().to_string();
        let channel = lines
            .next()
            .unwrap()
            .trim()
            .split(",")
            .map(|s| s.to_string())
            .collect();
        Ok(Self {
            agrCtlRSSI,
            agrExtRSSI,
            agrCtlNoise,
            agrExtNoise,
            state,
            op_mode,
            lastTxRate,
            maxRate,
            lastAssocStatus,
            auth,
            link_auth,
            BSSID,
            SSID,
            MCS,
            guardInterval,
            NSS,
            channel,
        })
    }
}

const STORAGE_PATH: &str = "./storage.persy";

struct MyState(Persy);

fn main() {
    tauri::Builder::default()
        .manage(MyState(
            Persy::open_or_create_with(STORAGE_PATH, PersyConfig::new(), |persy| Ok(()))
                .expect("Error opening or creating persy"),
        ))
        .system_tray(new_sys_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a left click");
            }
            SystemTrayEvent::RightClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
              match id.as_str() {
                "quit" => {
                  std::process::exit(0);
                }
                "hide" => {
                  let window = app.get_window("main").unwrap();
                  window.hide().unwrap();
                }
                _ => {}
              }
            }
            _ => {}
          })
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            main_window.open_devtools();
            let state: MyState = MyState(app.state::<MyState>().0.clone());
            spawn(move || loop {
                sleep(Duration::from_secs(1));
                let wifi_info = get_current_wifi();
                let wifi_list = get_wifi_list(wifi_info.clone(), false, &state.0);
                let dropped = has_signal_level_dropped(&wifi_list, wifi_info.SSID.as_str());

                let mut backup_wifi_list = get_wifi_list(wifi_info.clone(), true, &state.0);
                if dropped && !backup_wifi_list.is_empty() {
                    backup_wifi_list.sort_by(|a, b| a.signal_level.cmp(&b.signal_level));
                    let strongest_backup_wifi = backup_wifi_list.last().unwrap();
                    let wifi_password = get_wifi_password(&strongest_backup_wifi.ssid)
                        .expect("Error getting wifi password");
                    connect_to_wifi(&strongest_backup_wifi.ssid, &wifi_password);
                } else {
                    main_window
                        .emit_all(
                            "saved_wifi_list",
                            Payload {
                                wifi_list: get_wifi_list(wifi_info.clone(), true, &state.0),
                            },
                        )
                        .expect("failed to emit event");
                    main_window
                        .emit_all(
                            "available_wifi_list",
                            Payload {
                                wifi_list: wifi_list,
                            },
                        )
                        .expect("failed to emit event");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            invoke_wifi_list,
            get_current_wifi,
            toggle_backup_wifi
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(async)]
fn invoke_wifi_list(
    current_wifi_info: WifiInfo,
    filter: bool,
    state: tauri::State<MyState>,
) -> Vec<WiFi> {
    let persy = &state.0;
    get_wifi_list(get_current_wifi(), filter, persy)
}

fn get_wifi_list(current_wifi_info: WifiInfo, filter: bool, persy: &Persy) -> Vec<WiFi> {
    let connected = |wifi: &Wifi| {
        wifi.ssid == current_wifi_info.SSID && wifi.channel == current_wifi_info.channel[0]
    };
    let wifi_list: Vec<WiFi> = wifiscanner::scan()
        .expect("Error retrieving available wifi's")
        .into_iter()
        .map(|wifi| WiFi::new(&wifi, connected(&wifi)))
        .collect();

    if filter {
        let saved_ssids: Vec<String> = get_ssids(persy)
            .unwrap_or(Vec::new())
            .into_iter()
            .map(|s| s.0)
            .collect();
        wifi_list
            .into_iter()
            .filter(|wifi| {
                saved_ssids.contains(&wifi.ssid) && &wifi.ssid != &current_wifi_info.SSID
            })
            .collect()
    } else {
        wifi_list
    }
}

#[tauri::command(async)]
fn get_current_wifi() -> WifiInfo {
    use std::process::{Command, Stdio};

    let raw_info_child = Command::new(
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
    )
    .arg("-I")
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to start echo process");

    let echo_out = raw_info_child.stdout.expect("Failed to open echo stdout");

    let mut sed_child = Command::new("sed")
        .arg("s/.*://")
        .stdin(Stdio::from(echo_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start sed process");

    let output = sed_child.wait_with_output().expect("Failed to wait on sed");

    let data = String::from_utf8_lossy(&output.stdout);

    WifiInfo::from_str(&data).unwrap()
}

#[tauri::command(async)]
fn toggle_backup_wifi(
    ssid: &str,
    password: Option<&str>,
    state: tauri::State<MyState>,
) -> Result<(), String> {
    let service = "wifi_backup";
    let entry = keyring::Entry::new(&service, &ssid);
    let persy = &state.0;

    if let Some(password) = password {
        entry.set_password(&password).map_err(|e| e.to_string())?;
        return save_to_store(persy, ssid);
    } else {
        entry.delete_password().map_err(|e| e.to_string())?;
        return delete_from_store(persy, ssid);
    }
}

fn get_wifi_password(ssid: &str) -> Result<String, keyring::Error> {
    let service = "wifi_backup";
    let entry = keyring::Entry::new(&service, &ssid);
    entry.get_password()
}

fn save_to_store(persy: &Persy, ssid: &str) -> Result<(), String> {
    let mut tx = persy.begin().map_err(|e| e.to_string())?;
    if !tx.exists_segment("ssids").map_err(|e| e.to_string())? {
        tx.create_segment("ssids").map_err(|e| e.to_string())?;
    }
    let saved_ssids: Vec<String> = get_ssids(persy)?.into_iter().map(|s| s.0.clone()).collect();
    //Prepere some raw data
    let data = ssid.as_bytes();
    if !saved_ssids.contains(&ssid.to_string()) {
        //Insert the data inside the segment with the current tx.
        let id = tx.insert("ssids", &data).map_err(|e| e.to_string())?;
        //Commit the tx.
        let prepared = tx.prepare().map_err(|e| e.to_string())?;
        prepared.commit().map_err(|e| e.to_string())?;
    };
    Ok(())
}

fn get_ssids(persy: &Persy) -> Result<Vec<(String, PersyId)>, String> {
    persy
        .scan("ssids")
        .map_err(|s| s.to_string())?
        .into_iter()
        .map(|(id, data)| {
            let ssid = String::from_utf8(data).map_err(|e| e.to_string())?;
            Ok((ssid, id))
        })
        .collect()
}

fn delete_from_store(persy: &Persy, ssid: &str) -> Result<(), String> {
    let mut id = None;
    for (read_id, content) in persy.scan("ssids").map_err(|e| e.to_string())? {
        if content == ssid.as_bytes() {
            id = Some(read_id);
            break;
        }
    }

    let mut tx = persy.begin().map_err(|e| e.to_string())?;
    // delete the record
    tx.delete("ssids", &id.unwrap())
        .map_err(|e| e.to_string())?;
    //Commit the tx.
    let prepared = tx.prepare().map_err(|e| e.to_string())?;
    prepared.commit().map_err(|e| e.to_string())?;

    Ok(())
}

fn has_signal_level_dropped(wifi_list: &Vec<WiFi>, ssid: &str) -> bool {
    let connected_wifi = wifi_list.into_iter().find(|wifi| wifi.ssid == ssid);
    match connected_wifi {
        Some(wifi) => wifi.signal_level.parse::<i32>().unwrap() < -75,
        None => false,
    }
}

fn connect_to_wifi(ssid: &str, password: &str) -> Result<(), WifiConnectionError> {
    let config = Some(Config {
        interface: Some("en0"),
    });
    let mut wifi = WifiClient::new(config);
    println!("Connecting to {}, with {}", ssid, password);

    match wifi.connect(ssid, password) {
        Ok(result) => println!(
            "{}",
            if result == true {
                "Connection Successfull."
            } else {
                "Invalid password."
            }
        ),
        Err(err) => println!("The following error occurred: {:?}", err),
    }

    Ok(())
}

fn new_sys_tray() -> SystemTray {
    use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};

    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    SystemTray::new().with_menu(tray_menu)
}
