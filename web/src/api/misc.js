import instance from "@/utils/request.js";

export const miscVersionListRequest = () => instance.post('/misc/version-list', {})
