export const generateRandomStr = (length = 8) => {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  const charactersLength = characters.length;

  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }

  return result;
}

export const showFileSize = (size) => {
  if (size > (1024 * 1024)) {
    return `${(size / 1024 / 1024).toFixed(2)} MB`
  } else if (size > 1024) {
    return `${(size / 1024).toFixed(2)} KB`
  } else {
    return `${size} Bytes`
  }
}

export const showTime = (timestamp) => {
  return new Date(timestamp * 1000).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  });
}
