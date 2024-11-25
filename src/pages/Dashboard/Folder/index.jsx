import React, {useEffect, useState} from 'react';
import FileBreadcrumb from "@/components/FileBreadcrumb/index.jsx";
import {getFileListRequest} from "@/api/fileManager.js";
import TileViewFileExplorer from "@/components/TileViewFileExplorer/index.jsx";

const Index = () => {
  const [path, setPath] = useState([]);
  const [fileList, setFileList] = useState([])

  const getFileList = async () => {
    const {code, msg, data} = await getFileListRequest(path.join('/'));
    if (code === 1) {
      setFileList(data.files)
    }
  }

  const handlerNextPath = (item) => {
    setPath(prev => [...prev, item.name]);
  }

  const handlerBreadcrumb = (index) => {
    setPath(prev => prev.slice(0, index));
  }

  useEffect(() => {
    getFileList()
  }, [path]);

  return (
    <>
      <div>
        <FileBreadcrumb path={path} handlerBreadcrumb={handlerBreadcrumb}/>
        <TileViewFileExplorer items={fileList} handlerNextPath={handlerNextPath}/>
      </div>
    </>
  );
};

export default Index;
