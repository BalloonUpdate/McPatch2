import React from 'react';

const Index = ({path, handlerBreadcrumb}) => {

  const items = ['首页', ...path]
  return (
    <>
      <div className="h-16 pr-4 pl-4 border rounded-lg border-light shadow-1">
        <div className="h-8 flex items-center text-indigo-600 font-bold text-base">工作目录</div>
        <ul className="h-8 flex items-center">
          {
            items.map((item, index) => {
              return (
                <li key={index} className="flex items-center cursor-default">
                  {
                    items.length - 1 !== index ?
                      <div className="text-base text-gray-800 font-medium">
                        <button onClick={() => handlerBreadcrumb(index)}>{item}</button>
                        <span className="px-3 text-body-color">{" / "}</span>
                      </div> :
                      <div className="text-base text-gray-400 font-medium">
                        {item}
                      </div>
                  }
                </li>
              )
            })
          }
        </ul>
      </div>
    </>
  );
};

export default Index;
