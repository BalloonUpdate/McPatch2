import instance from "@/utils/request.js";

export const userLoginRequest = (username, password) => instance.post('/user/login', {username, password})
