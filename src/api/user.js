import instance from "@/utils/request.js";

export const userLoginRequest = (username, password) => instance.post('/user/login', {username, password})

export const userSignOutRequest = () => instance.post('/user/logout', {})
