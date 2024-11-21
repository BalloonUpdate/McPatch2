import {createBrowserRouter} from "react-router-dom";
import Home from "@/pages/Hom/index.jsx";
import NotFound from "@/pages/NotFound/index.jsx";
import Dashboard from "@/pages/Dashboar/index.jsx";
import Overview from "@/pages/Dashboar/Overvie/index.jsx";
import Folder from "@/pages/Dashboar/Folde/index.jsx";
import Log from "@/pages/Dashboar/Lo/index.jsx";
import Help from "@/pages/Dashboar/Hel/index.jsx";
import Settings from "@/pages/Dashboar/Setting/index.jsx";
import Login from "@/pages/Logi/index.jsx";

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
