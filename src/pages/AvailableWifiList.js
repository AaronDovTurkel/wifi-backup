import './SavedBackupWifi.css';
import { useEffect, useState } from 'react';
import WifiList from '../components/WifiList.js';
import { listen } from '@tauri-apps/api/event'

function AvailableWifiList({save}) {

  const [wifiList, setWifiList] = useState([]);

  
  useEffect(() => {
    async function listeners(callback) {
      const unlisten = await listen('available_wifi_list', callback);
      return unlisten;
    }
    const unlisten = listeners((event) => {
      setWifiList(event.payload.wifi_list);
    });
    return function cleanup() {
      unlisten.then(unlisten => unlisten());
    }
  }, [setWifiList])

  return (
    <WifiList
      wifiList={wifiList}
      refresh={() => ([])} 
      placeHolder="There are no available backup WiFi connections..."
      save={(ssid) => save(ssid)}
    />
  )
}

export default AvailableWifiList;