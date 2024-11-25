import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import '@/assets/index.css'

import {RouterProvider} from "react-router-dom";
import {Provider} from "react-redux";

import router from "@/router/index.jsx";
import store from "@/store/index.js";
import {ConfigProvider} from "antd";

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <ConfigProvider theme={{token: {colorPrimary: '#4f46e5'}}}>
      <Provider store={store}>
        <RouterProvider router={router}/>
      </Provider>
    </ConfigProvider>
  </StrictMode>
)
