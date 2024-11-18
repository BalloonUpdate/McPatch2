import {createSlice} from "@reduxjs/toolkit";
import {userLoginRequest} from "@/api/user.js";

const userStore = createSlice({
  name: "user",
  initialState: {
    token: localStorage.getItem('token') || ''
  },
  reducers: {
    setUser(state, action) {
      state.token = action.payload.token;
      localStorage.setItem('token', action.payload.token);
    },
    clearUser(state) {
      state.token = '';
      localStorage.removeItem('token');
    }
  }
})

const {setUser, clearUser} = userStore.actions;
const userLogin = (loginForm) => {
  return async (dispatch) => {
    const {code, msg, data} = await userLoginRequest(loginForm.username, loginForm.password);
    if (code === 1) {
      dispatch(setUser(data));
      return true
    } else {
      return msg
    }
  }
}
export {userLogin, clearUser}

const reducer = userStore.reducer
export default reducer
