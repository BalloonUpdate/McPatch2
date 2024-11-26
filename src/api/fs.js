import instance from "@/utils/request.js";
import axios from "axios";

export const fsDiskInfoRequest = () => instance.post('/fs/disk-info', {})

export const fsListRequest = (path = '') => instance.post('/fs/list', {path})

export const fsMakeDirectoryRequest = (path = '') => instance.post('/fs/make-directory', {path})

export const fsUploadRequest = (path = '', file, onProgress) => {
  const formData = new FormData();
  formData.append('path', path);
  formData.append('file', file);

  return axios.post(`${import.meta.env.VITE_API_URL}/fs/upload`, formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
    onUploadProgress: (event) => {
      let percent = Math.floor((event.loaded / event.total) * 100);
      onProgress({percent});
    },
  });
}
