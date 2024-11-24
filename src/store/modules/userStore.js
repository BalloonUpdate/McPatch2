import {createSlice} from "@reduxjs/toolkit";
import {userLoginRequest} from "@/api/user.js";

const userStore = createSlice({
  name: "user",
  initialState: {
    token: localStorage.getItem('token') || ''
  },
  reducers: {
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

const {setToken, clearToken} = userStore.actions;
const userLogin = (username, password) => {
  return async (dispatch) => {
    const {code, msg, data} = await userLoginRequest(username, password);
    const flag = code === 0
    if (flag) {
      dispatch(setToken(data))
    }

    return {flag, msg}
  }
}
export {userLogin, clearToken}

const reducer = userStore.reducer
export default reducer
