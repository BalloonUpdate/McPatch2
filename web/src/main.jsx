import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import '@/assets/index.css'

import {RouterProvider} from "react-router-dom";
import {Provider} from "react-redux";

import router from "@/router/index.jsx";
import store from "@/store/index.js";

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <Provider store={store}>
      <RouterProvider router={router}/>
    </Provider>
  </StrictMode>
)
