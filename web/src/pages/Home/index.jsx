import {useNavigate} from "react-router-dom";
import {useSelector} from "react-redux";
import {theme} from "antd";

const {useToken} = theme;

const Index = () => {

  const user = useSelector(state => state.user)
  const navigate = useNavigate();
  const {token} = useToken();

  const checkStatus = () => {
    if (user.token) {
      navigate("/dashboard")
    } else {
      navigate("/login")
    }
  }

  return (
    <>
      <div className="w-screen h-screen flex flex-col justify-center items-center space-y-4">
        <h2
          className="text-transparent bg-clip-text bg-gradient-to-r from-[#4F46E5] to-[#E114E5] text-4xl font-extrabold md:text-5xl">
          McPatch
        </h2>
        <p className={`max-w-2xl mx-auto text-center dark:text-white`}>
          McPatch 是一个给 Minecraft 客户端做文件更新的独立应用程序.只要你想,你可以通过这个程序向你服务器的玩家提供一切内容.
        </p>
        <button
          onClick={() => checkStatus()}
          className="px-6 py-3.5 text-white bg-indigo-600 rounded-full duration-150 hover:bg-indigo-500 active:bg-indigo-700">
          即刻开始!
        </button>
      </div>
    </>
  );
};

export default Index;
