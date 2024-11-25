import instance from "@/utils/request.js";

export const getDiskInfoRequest = () => instance.post('/fs/disk-info', {})

export const getFileListRequest = (path = '') => instance.post('/fs/list', {path})
