import instance from "@/utils/request.js";
import axios from "axios";
import store from "@/store/index.js";

export const fsDiskInfoRequest = () => instance.post('/fs/disk-info', {})

export const fsListRequest = (path = '') => instance.post('/fs/list', {path})

export const fsMakeDirectoryRequest = (path = '') => instance.post('/fs/make-directory', {path})

export const fsDeleteRequest = (path = '') => instance.post('/fs/delete', {path})

export const fsSignFileRequest = (path = '') => instance.post('/fs/sign-file', {path})

export const fsUploadRequest = (path = '', file, onProgress) => {
  return axios.post(`${import.meta.env.VITE_API_URL}/fs/upload`, file, {
    headers: {
      'Token': store.getState().user.token,
      'Content-Type': 'application/octet-stream',
      'Path': encodeURIComponent(path)
    },
    onUploadProgress: (event) => {
      let percent = Math.floor((event.loaded / event.total) * 100);
      onProgress({percent});
    },
  });
}
