import './WifiList.css'
import { useEffect, useState } from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faArrowsRotate, faTrashCan } from '@fortawesome/free-solid-svg-icons'

function WifiList({wifiList, del, add, save, search = true, placeHolder = "There are no available wifi connections."}) {
    const [searchTerm, setSearchTerm] = useState('');
    const [filteredWifiList, setFilteredWifiList] = useState(wifiList);
    useEffect(() => {
        const list = searchTerm ? wifiList.filter(wifi => wifi.ssid.toLowerCase().includes(searchTerm.toLowerCase())) : wifiList;
         setFilteredWifiList(list.sort((a, b) => b.signal_level - a.signal_level));
    }, [searchTerm, wifiList])

    return (
    <div className="wifi-list">
        {search && <div className="wifi-list_search-wrap">
            <input className="wifi-list_input" value={searchTerm} onChange={e => setSearchTerm(e.target.value)} placeholder="Search for wifi names" />
        </div>}
        <ul className="wifi-list_list">
            {!filteredWifiList.length 
                    ? searchTerm ? 'There are no available wifi\'s that match that search' : placeHolder
                    : filteredWifiList.map(wifi => 
                        <li className={`wifi-list_list-item ${del ? 'no-hover' : ''}`} onClick={() => save && save(wifi.ssid)}>
                            <span>Name: {wifi.ssid}</span>
                            {wifi.connected && <span className="wifi-list_span-connected">Connected</span>}
                            <div className="wifi-list_actions">
                                <span>Stength: {wifi.signal_level}</span>
                                {del && <FontAwesomeIcon className="wifi-list_del" icon={faTrashCan} onClick={() => del(wifi.ssid)} />}
                            </div>
                        </li>
                    )
            }
        </ul>
        {add && <button className="wifi-list_add-new" onClick={() => add()}>Add a new back up WiFi</button>}
    </div>
    )
}

export default WifiList;
