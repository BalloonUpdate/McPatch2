import instance from "@/utils/request.js";

export const userLoginRequest = (username, password) => instance.post('/user/login', {username, password})

export const userSignOutRequest = () => instance.post('/user/logout', {})

export const userChangeUsernameRequest = (newUsername) => instance.post('/user/change-username', {
  new_username: newUsername
})

export const userChangePasswordRequest = (oldPassword, newPassword) => instance.post('/user/change-password', {
  old_password: oldPassword,
  new_password: newPassword
})
