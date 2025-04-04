import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

// 天気データの型定義
interface WeatherData {
  description: string;
  temperature: number | string;
  humidity: number | string;
  icon: string;
  weather_code?: string;
}

function App() {
  const [currentTime, setCurrentTime] = useState<string>('読込中...');
  const [notifyEnabled, setNotifyEnabled] = useState<boolean>(false);
  const [weather, setWeather] = useState<WeatherData>({
    description: '読込中...',
    temperature: '--',
    humidity: '--',
    icon: 'unknown',
  });

  // バックエンドから時刻を取得する関数
  async function fetchTime(): Promise<string | null> {
    try {
      const time = await invoke<string>('get_current_time');
      setCurrentTime(time);
      return time;
    } catch (error) {
      console.error('時刻取得エラー:', error);
      setCurrentTime('エラーが発生しました');
      return null;
    }
  }

  // 東京の天気を取得する関数
  async function fetchWeather(): Promise<void> {
    try {
      // 気象庁APIからデータを取得
      const weatherData = await invoke<WeatherData>('get_tokyo_weather');
      setWeather(weatherData);
    } catch (error) {
      console.error('天気取得エラー:', error);
      // エラー時はデモデータを使用
      try {
        const demoData = await invoke<WeatherData>('get_tokyo_weather_demo');
        setWeather(demoData);
      } catch (fallbackError) {
        setWeather({
          description: 'エラーが発生しました',
          temperature: '--',
          humidity: '--',
          icon: 'error'
        });
      }
    }
  }

  // 通知を手動送信する関数
  async function sendNotification() {
    const time = await fetchTime();
    if (time) {
      try {
        await invoke('send_notification', {
          title: '現在時刻',
          body: `現在の時刻は ${time} です`
        });
      } catch (error) {
        console.error('通知エラー:', error);
      }
    }
  }

  // 通知設定の切り替え
  async function toggleNotification(e: React.ChangeEvent<HTMLInputElement>) {
    const isChecked = e.target.checked;
    setNotifyEnabled(isChecked);
    
    try {
      // バックエンドの通知設定を更新
      await invoke('toggle_notification', { enabled: isChecked });
      
      // チェックされた場合は最初の通知を即座に送信
      if (isChecked) {
        sendNotification();
      }
    } catch (error) {
      console.error('通知設定エラー:', error);
    }
  }

  // バックエンドから通知設定状態を取得
  async function fetchNotificationState() {
    try {
      const enabled = await invoke<boolean>('get_notification_state');
      setNotifyEnabled(enabled);
    } catch (error) {
      console.error('通知設定状態の取得エラー:', error);
    }
  }

  useEffect(() => {
    // 初回読み込み時に時刻、通知設定、天気を取得
    fetchTime();
    fetchNotificationState();
    fetchWeather();
    
    // 1秒ごとに時刻を更新
    const timeInterval = setInterval(fetchTime, 1000);
    
    // 30分ごとに天気情報を更新
    const weatherInterval = setInterval(fetchWeather, 1800000);
    
    // クリーンアップ
    return () => {
      clearInterval(timeInterval);
      clearInterval(weatherInterval);
    };
  }, []);

  // 天気アイコンのURLを生成する関数
  function getWeatherIconUrl(iconCode: string) {
    if (iconCode === 'unknown' || iconCode === 'error') {
      return undefined;
    }
    return `https://openweathermap.org/img/wn/${iconCode}@2x.png`;
  }

  return (
    <div className="container">
      <h1>Tauri 時計アプリ</h1>
      
      <div className="time-display">
        <h2>現在時刻:</h2>
        <p className="time">{currentTime}</p>
      </div>
      
      <div className="weather-display">
        <h2>東京の天気:</h2>
        <div className="weather-info">
          {weather.icon !== 'unknown' && weather.icon !== 'error' && (
            <img 
              src={getWeatherIconUrl(weather.icon)} 
              alt={weather.description}
              className="weather-icon"
            />
          )}
          <div className="weather-details">
            <p className="weather-desc">{weather.description}</p>
            <p className="weather-temp">{weather.temperature !== '--' ? `${weather.temperature}°C` : '--°C'}</p>
            <p className="weather-humidity">湿度: {weather.humidity !== '--' ? `${weather.humidity}%` : '--%'}</p>
            {weather.weather_code && (
              <p className="weather-code">気象庁コード: {weather.weather_code}</p>
            )}
          </div>
        </div>
        <button onClick={fetchWeather} className="weather-button">
          天気更新
        </button>
      </div>
      
      <div className="notification-setting">
        <label>
          <input 
            type="checkbox" 
            checked={notifyEnabled} 
            onChange={toggleNotification} 
          />
          5分刻みの時刻（00, 05, 10, 15...分）で通知を受け取る
        </label>
      </div>
      
      <button onClick={sendNotification} className="notify-button">
        今すぐ通知を送信
      </button>
    </div>
  );
}

export default App;
