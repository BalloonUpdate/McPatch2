import {configureStore} from "@reduxjs/toolkit";
import userReducer from "@/store/modules/userStore.js";

const store = configureStore({
  reducer: {
    user: userReducer,
  }
})

export default store
