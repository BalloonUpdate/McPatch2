import instance from "@/utils/request.js";

export const terminalFullRequest = () => instance.post('/terminal/full', {})

export const terminalMoreRequest = () => instance.post('/terminal/more', {})
