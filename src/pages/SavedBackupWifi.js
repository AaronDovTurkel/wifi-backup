import './SavedBackupWifi.css';
import { useEffect, useState } from 'react';
import WifiList from '../components/WifiList.js';
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api';
import { toast } from 'react-toastify';

function SavedBackupWifi({add}) {

  const [wifiList, setWifiList] = useState([]);

  
  useEffect(() => {
    async function listeners(callback) {
      const unlisten = await listen('saved_wifi_list', callback);
      return unlisten;
    }
    const unlisten = listeners((event) => {
      setWifiList(event.payload.wifi_list);
    });
    return function cleanup() {
      unlisten.then(unlisten => unlisten());
    }
  }, [setWifiList])

  function del(ssid) {
      invoke('toggle_backup_wifi', { ssid }).then(
          res => {
              console.log(res);
              toast.success('Wifi connection deleted');
          },
          err => {
              console.log(err);
              toast.error(err.message);
          }
      )
  }
  

  return (
    <WifiList
      wifiList={wifiList}
      del={del} 
      placeHolder="You dont have any saved backup WiFi connections..."
      add={() => add()}
    />
  )
}

export default SavedBackupWifi;
