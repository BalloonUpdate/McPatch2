import {createBrowserRouter} from "react-router-dom";
import Home from "@/pages/home/index.jsx";
import NotFound from "@/pages/not-found/index.jsx";
import Dashboard from "@/pages/dashboard/index.jsx";
import Overview from "@/pages/dashboard/overview/index.jsx";
import Folder from "@/pages/dashboard/folder/index.jsx";
import Log from "@/pages/dashboard/log/index.jsx";
import Help from "@/pages/dashboard/help/index.jsx";
import Settings from "@/pages/dashboard/settings/index.jsx";
import Login from "@/pages/login/index.jsx";

const router = createBrowserRouter([
  {
    path: '/',
    element: <Home/>
  },
  {
    path: '/login',
    element: <Login/>
  },
  {
    path: '/dashboard',
    element: <Dashboard/>,
    children: [
      {
        index: true,
        element: <Overview/>
      },
      {
        path: 'folder',
        element: <Folder/>
      },
      {
        path: 'log',
        element: <Log/>
      },
      {
        path: 'help',
        element: <Help/>
      },
      {
        path: 'settings',
        element: <Settings/>
      }
    ]
  },
  {
    path: '*',
    element: <NotFound/>
  }
])

export default router
