import axios from "axios"
import store from "@/store/index.js";

const instance = axios.create({
  baseURL: window.location.origin + '/api',
  timeout: 10000
})

instance.interceptors.request.use(
  config => {
    config.headers.setContentType("application/json")

    const token = store.getState().user.token;
    if (token) {
      config.headers.Authorization = 'Bearer ' + token
    }
    return config
  },
  error => {
    return error
  }
)

instance.interceptors.response.use(
  res => {
    if (res.status === 200) {
      return res.data
    }
  },
  error => {
    return error
  }
)

export default instance
