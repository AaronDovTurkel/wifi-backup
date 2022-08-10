import { useState } from 'react';
import './App.css';
import SavedBackupWifi from './pages/SavedBackupWifi.js'
import AvailableWifiList from './pages/AvailableWifiList.js'
import SaveWifi from "./pages/SaveWifi";
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faLeftLong } from '@fortawesome/free-solid-svg-icons'
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';



function App() {

  const [selectedSSID, setSelectedSSID] = useState(null);
  const [page, setPage] = useState('saved');
  const withBackButton = ['available', 'save'].includes(page);
  const title = {
    saved: 'Saved WiFi',
    available: 'Available WiFi',
    save: 'Save WiFi'
  }[page];
  const currentPage = (page, ssid) => ({
    saved: <SavedBackupWifi add={() => setPage('available')} />,
    available: <AvailableWifiList togglePage={() => setPage('saved')} save={(ssid) => {
      console.log({ssid})
      setSelectedSSID(ssid);
      setTimeout(() => setPage('save'), 0);
    }} />,
    save: SaveWifi({ssid: selectedSSID, togglePage: () => setPage('saved')})
  }[page]);

  return (
    <div className="app">
      <header className="app_header">
        <img className="app_logo" src="./assets/logo.png" alt="logo" />
      </header>
      <main className="app_main">
        <h1 className="app_title">
          {withBackButton 
            ? <FontAwesomeIcon className="app_back" icon={faLeftLong} onClick={() => setPage(page === 'save' ? 'available' : 'saved')} />
            : <div></div>
          }
          {title}
        </h1>
        {currentPage(page, selectedSSID)}
      </main>
      <footer className="app_footer">Made with 🍻 by <a className='app_link' href="https://aarondovturkel.com" target="_blank" rel="noreferrer">Aaron Dov Turkel</a></footer>
      <ToastContainer />
    </div>
  );
}

export default App;
