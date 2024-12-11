import React, {useEffect, useState} from 'react';
import {Outlet} from "react-router-dom";
import {ConfigProvider, FloatButton, theme} from "antd";
import {MoonStar, Sun} from "lucide-react";

const App = () => {

  const [darkMode, setDarkMode] = useState(localStorage.getItem("darkMode") === "true");

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('darkMode', darkMode.toString());
  }, [darkMode]);

  return (
    <>
      <ConfigProvider
        theme={{token: {colorPrimary: '#4f46e5'}, algorithm: darkMode ? theme.darkAlgorithm : theme.defaultAlgorithm}}>
        <div className="dark:bg-gray-950">
          <Outlet/>
          <FloatButton
            icon={darkMode ? <Sun className="w-full h-full"/> : <MoonStar className="w-full h-full"/>}
            tooltip={<div>深色模式</div>}
            onClick={() => setDarkMode(!darkMode)}/>
        </div>
      </ConfigProvider>
    </>
  );
};

export default App;
