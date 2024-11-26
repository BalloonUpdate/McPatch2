import instance from "@/utils/request.js";

export const fsDiskInfoRequest = () => instance.post('/fs/disk-info', {})

export const fsListRequest = (path = '') => instance.post('/fs/list', {path})
