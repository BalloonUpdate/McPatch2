import instance from "@/utils/request.js";

export const taskPackRequest = (label, changeLogs) => instance.post('/task/pack', {
  label: label,
  change_logs: changeLogs
})

export const taskCombineRequest = () => instance.post('/task/combine', {})

export const taskTestRequest = () => instance.post('/task/test', {})

export const taskRevertRequest = () => instance.post('/task/revert', {})
