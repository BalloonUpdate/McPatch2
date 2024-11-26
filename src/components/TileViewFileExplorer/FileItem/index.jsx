import React from 'react';

const Index = ({item}) => {

  const getBgColor = (item) => {
    if (item.state === 'added') return 'hover:bg-green-100';
    if (item.state === 'modified') return 'hover:bg-yellow-100';
    if (item.state === 'missing') return 'hover:bg-red-100';
    if (item.state === 'gone') return 'hover:bg-cyan-100';
    if (item.state === 'come') return 'hover:bg-violet-100';
    return 'hover:bg-gray-200';
  };

  const getTextColor = (item) => {
    if (item.state === 'added') return 'text-green-500';
    if (item.state === 'modified') return 'text-yellow-500';
    if (item.state === 'missing') return 'text-red-500';
    if (item.state === 'gone') return 'text-cyan-500';
    if (item.state === 'come') return 'text-violet-500';
    return 'text-gray-500';
  };

  return (
    <>
      <div
        className={`w-24 h-24 flex flex-col justify-center items-center cursor-pointer duration-200 select-none ${getBgColor(item)}`}>
        <div className="max-w-20 text-3xl">
          {item.is_directory ? '📁' : '📄'}
        </div>
        <div
          className={`max-w-20 whitespace-nowrap overflow-hidden overflow-ellipsis ${getTextColor(item)}`}>{item.name}</div>
      </div>
    </>
  );
};

export default Index;
