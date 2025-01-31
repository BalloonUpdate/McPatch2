import {createSlice} from "@reduxjs/toolkit";
import {userLoginRequest} from "@/api/user.js";

const userStore = createSlice({
  name: "user",
  initialState: {
    username: localStorage.getItem('username') || '',
    token: localStorage.getItem('token') || ''
  },
  reducers: {
    setUser: (state, action) => {
      state.username = action.payload.username;
      localStorage.setItem('username', action.payload.username);
    },
    setToken(state, action) {
      state.token = action.payload.token;
      localStorage.setItem('token', action.payload.token);
    },
    clearToken(state) {
      state.token = '';
      localStorage.removeItem('token');
    }
  }
})

const {setUser, setToken, clearToken} = userStore.actions;
const userLogin = (username, password) => {
  return async (dispatch) => {
    const {code, msg, data} = await userLoginRequest(username, password);
    const flag = code === 1
    if (flag) {
      dispatch(setUser({username}))
      dispatch(setToken(data))
    }

    return {flag, msg}
  }
}
export {setUser, userLogin, clearToken}

const reducer = userStore.reducer
export default reducer
