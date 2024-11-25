import {Outlet, useLocation, useNavigate} from "react-router-dom";
import {AppWindow, CircleHelp, CircleUserRound, Folder, LogOut, ScrollText, Settings} from "lucide-react";
import {userSignOutRequest} from "@/api/user.js";
import {message} from "antd";
import {useDispatch, useSelector} from "react-redux";
import {clearToken} from "@/store/modules/userStore.js";
import {useEffect} from "react";

const Index = () => {

  const navigate = useNavigate();
  const location = useLocation();
  const user = useSelector((state) => state.user);
  const dispatch = useDispatch();
  const [messageApi, contextHolder] = message.useMessage();

  useEffect(() => {
    if (!user.token) {
      location.href = "/login";
    }
  }, []);

  const signOut = async () => {
    const {code, msg, data} = await userSignOutRequest()
    if (code === 1) {
      dispatch(clearToken())
      navigate('/login?type=signOut');
    } else {
      messageApi.error(msg)
    }
  }

  const navs = [
    {
      nav: '/dashboard',
      name: '概览',
      icon: <AppWindow size={16} strokeWidth={1.5}/>
    },
    {
      nav: '/dashboard/folder',
      name: '文件',
      icon: <Folder size={16} strokeWidth={1.5}/>
    },
    {
      nav: '/dashboard/log',
      name: '日志',
      icon: <ScrollText size={16} strokeWidth={1.5}/>
    }
  ]

  const navsFooter = [
    {
      nav: '/dashboard/help',
      name: '帮助',
      icon: <CircleHelp size={16} strokeWidth={1.5}/>
    },
    {
      nav: '/dashboard/settings',
      name: '设置',
      icon: <Settings size={16} strokeWidth={1.5}/>
    }
  ]

  return (
    <>
      {contextHolder}
      <div className="flex">
        <div
          className="fixed top-0 left-0 w-full h-full border-r bg-white space-y-8 sm:w-60">
          <div className="flex flex-col h-full">
            <div className='h-20 flex justify-center items-center px-8'>
              <div className='flex-none cursor-pointer' onClick={() => navigate('/')}>
                <div className="text-3xl font-bold text-indigo-600">McPatch</div>
              </div>
            </div>
            <div className="flex-1 flex flex-col h-full overflow-auto">
              <ul className="px-4 text-sm font-medium flex-1">
                {
                  navs.map((item, idx) => {
                    const isActive = location.pathname === item.nav
                    return (
                      <li key={idx}>
                        <div onClick={() => navigate(item.nav)}
                             className={`flex items-center gap-x-2 text-gray-600 p-2 rounded-lg cursor-pointer ${isActive ? 'bg-gray-100' : 'hover:bg-gray-50 active:bg-gray-100 duration-150'}`}>
                          <div className="text-gray-500">{item.icon}</div>
                          {item.name}
                        </div>
                      </li>
                    )
                  })
                }
              </ul>
              <div>
                <ul className="px-4 pb-4 text-sm font-medium">
                  {
                    navsFooter.map((item, idx) => {
                      const isActive = location.pathname === item.nav
                      return (
                        <li key={idx}>
                          <div onClick={() => navigate(item.nav)}
                               className={`flex items-center gap-x-2 text-gray-600 p-2 rounded-lg cursor-pointer ${isActive ? 'bg-gray-100' : 'hover:bg-gray-50 active:bg-gray-100 duration-150'}`}>
                            <div className="text-gray-500">{item.icon}</div>
                            {item.name}
                          </div>
                        </li>
                      )
                    })
                  }
                  <li>
                    <div
                      onClick={() => signOut()}
                      className={`flex items-center gap-x-2 text-gray-600 p-2 rounded-lg cursor-pointer hover:bg-gray-50 active:bg-gray-100 duration-150}`}>
                      <div className="text-gray-500"><LogOut size={16} strokeWidth={1.5}/></div>
                      退出登录
                    </div>
                  </li>
                </ul>
                <div className="py-4 px-4 border-t">
                  <div className="flex items-center gap-x-4">
                    {/*<img src="" className="w-12 h-12 rounded-full"/>*/}
                    <CircleUserRound size={40} strokeWidth={1.0}/>
                    <div>
                      <span className="block text-gray-700 text-sm font-semibold">ADMIN</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="ml-60 p-10 flex-grow">
          <Outlet/>
        </div>
      </div>
    </>
  );
};

export default Index;
