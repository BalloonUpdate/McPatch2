const Index = () => {

  const login = (e) => {
    e.preventDefault()
    const username = e.target[0].value
    const password = e.target[1].value
    const remember = e.target[2].checked

    console.log(username)
    console.log(password)
    console.log(remember)
  }

  return (
    <>
      <div className="w-full h-screen flex flex-col items-center justify-center px-4">
        <div className="max-w-sm w-full text-gray-600 space-y-5">
          <div className="text-center pb-8">
            <div className="text-4xl text-indigo-600">McPatch</div>
          </div>
          <form
            onSubmit={login}
            className="space-y-5"
          >
            <div>
              <label className="font-medium">
                用户名
              </label>
              <input
                name="username"
                type="text"
                required
                className="w-full mt-2 px-3 py-2 text-gray-500 bg-transparent outline-none border focus:border-indigo-600 shadow-sm rounded-lg"
              />
            </div>
            <div>
              <label className="font-medium">
                密码
              </label>
              <input
                name="password"
                type="password"
                required
                className="w-full mt-2 px-3 py-2 text-gray-500 bg-transparent outline-none border focus:border-indigo-600 shadow-sm rounded-lg"
              />
            </div>
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-x-3">
                <input type="checkbox" id="remember-me-checkbox" className="checkbox-item peer hidden"
                       name="rememberMe"/>
                <label
                  htmlFor="remember-me-checkbox"
                  className="relative flex w-5 h-5 bg-white peer-checked:bg-indigo-600 rounded-md border ring-offset-2 ring-indigo-600 duration-150 peer-active:ring cursor-pointer after:absolute after:inset-x-0 after:top-[3px] after:m-auto after:w-1.5 after:h-2.5 after:border-r-2 after:border-b-2 after:border-white after:rotate-45"
                >
                </label>
                <span>记住密码</span>
              </div>
              <a href="javascript:void(0)" className="text-center text-indigo-600 hover:text-indigo-500">忘记密码?</a>
            </div>
            <button
              className="w-full px-4 py-2 text-white font-medium bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-600 rounded-lg duration-150"
              type="submit"
            >
              登录
            </button>
          </form>
        </div>
      </div>
    </>
  );
};

export default Index;
