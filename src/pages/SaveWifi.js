import { invoke } from '@tauri-apps/api';
import { useState } from 'react';
import { toast } from "react-toastify";

function SaveWifi({ togglePage, ssid }) {

    const [password, setPassword] = useState('');

    function save() {
        console.log(ssid, password);
        invoke('toggle_backup_wifi', { ssid, password }).then(
            res => {
                console.log(res);
                toast.success('Wifi connection saved');
                togglePage();
            },
            err => {
                console.log(err);
                toast.error(err.message);
            }
        )
    }

    return (
        <div className="save-wifi">
            <h2 className="save-wifi_header">{ssid}</h2>
            <input className="save-wifi_input" type="password" placeholder="Password" onChange={e => setPassword(e.target.value)} />
            <button className="save-wifi_button" onClick={() => save()}>Save</button>
        </div>
    )
}

export default SaveWifi;
