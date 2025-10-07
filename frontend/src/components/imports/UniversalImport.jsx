import { useState, useCallback } from 'react';

export default function UniversalImport() {
  const [dragOver, setDragOver] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState([]);
  const [selectedTemplate, setSelectedTemplate] = useState('custom');

  const acceptedFormats = ['.xlsx', '.csv', '.json', '.xml'];
  
  const templates = {
    custom: { name: 'Ръчно мапиране', icon: '⚙️', description: 'Свободно мапиране на полетата' },
    journal: { name: 'Дневник записи', icon: '📝', description: 'Стандартен формат за счетоводни записи' },
    accounts: { name: 'Сметкоплан', icon: '🗂️', description: 'Импорт на сметки и салда' },
    counterparts: { name: 'Контрагенти', icon: '👥', description: 'Списък с контрагенти' }
  };
  
  const features = [
    'Excel и CSV файлове',
    'JSON структуриран импорт',
    'Свободно мапиране на полета',
    'Bulk импорт на записи',
    'Валидация и предварителен преглед',
    'Шаблони за импорт'
  ];

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback((e) => {
    e.preventDefault();
    setDragOver(false);
    
    const files = Array.from(e.dataTransfer.files);
    const validFiles = files.filter(file => 
      acceptedFormats.some(format => file.name.toLowerCase().endsWith(format))
    );
    
    if (validFiles.length > 0) {
      setUploadedFiles(prev => [...prev, ...validFiles.map(file => ({
        name: file.name,
        size: file.size,
        type: file.type,
        status: 'pending',
        file: file,
        template: selectedTemplate
      }))]);
    }
  }, [selectedTemplate]);

  const handleFileSelect = useCallback((e) => {
    const files = Array.from(e.target.files);
    setUploadedFiles(prev => [...prev, ...files.map(file => ({
      name: file.name,
      size: file.size,
      type: file.type,
      status: 'pending',
      file: file,
      template: selectedTemplate
    }))]);
  }, [selectedTemplate]);

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const removeFile = (index) => {
    setUploadedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const startImport = async () => {
    if (uploadedFiles.length === 0) return;
    
    // TODO: Implement universal import logic
    console.log('Starting universal import with template:', selectedTemplate);
    console.log('Files:', uploadedFiles);
    
    // Simulate processing
    setUploadedFiles(prev => prev.map(file => ({ ...file, status: 'processing' })));
    
    setTimeout(() => {
      setUploadedFiles(prev => prev.map(file => ({ ...file, status: 'completed' })));
    }, 3000);
  };

  const downloadTemplate = (templateType) => {
    // TODO: Generate and download template file
    console.log('Downloading template:', templateType);
  };

  return (
    <div className="space-y-6">
      {/* Source Info */}
      <div className="flex items-start space-x-4">
        <div className="text-4xl">📊</div>
        <div>
          <h3 className="text-lg font-semibold text-gray-900">
            Универсален импорт
          </h3>
          <p className="text-gray-600 mb-3">
            Импорт от Excel, CSV и други формати с възможност за свободно мапиране
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {features.map((feature, index) => (
              <div key={index} className="flex items-center space-x-2">
                <span className="w-2 h-2 bg-green-400 rounded-full"></span>
                <span className="text-sm text-gray-700">{feature}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Template Selection */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h4 className="text-sm font-medium text-gray-900">Избор на шаблон:</h4>
          <button
            onClick={() => downloadTemplate(selectedTemplate)}
            className="text-xs text-blue-600 hover:text-blue-700 font-medium"
          >
            📥 Изтегли шаблон
          </button>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {Object.entries(templates).map(([key, template]) => (
            <button
              key={key}
              onClick={() => setSelectedTemplate(key)}
              className={`p-4 border-2 rounded-lg text-left transition-colors ${
                selectedTemplate === key
                  ? 'border-green-500 bg-green-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <div className="flex items-start space-x-3">
                <div className="text-2xl">{template.icon}</div>
                <div>
                  <div className="font-medium text-gray-900">{template.name}</div>
                  <div className="text-sm text-gray-600 mt-1">
                    {template.description}
                  </div>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Upload Area */}
      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragOver
            ? 'border-green-500 bg-green-50'
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="space-y-4">
          <div className="text-4xl">📊</div>
          <div>
            <p className="text-lg font-medium text-gray-900">
              Пуснете файловете за импорт тук или
            </p>
            <label className="cursor-pointer">
              <span className="text-green-600 hover:text-green-700 font-medium">
                изберете файлове
              </span>
              <input
                type="file"
                multiple
                accept={acceptedFormats.join(',')}
                onChange={handleFileSelect}
                className="hidden"
              />
            </label>
          </div>
          <p className="text-sm text-gray-500">
            Поддържани формати: {acceptedFormats.join(', ')}
          </p>
          <div className="text-xs text-gray-500">
            Максимум 20 файла, до 100MB всеки
          </div>
        </div>
      </div>

      {/* Uploaded Files */}
      {uploadedFiles.length > 0 && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-gray-900">
            Качени файлове ({uploadedFiles.length})
          </h4>
          <div className="space-y-2">
            {uploadedFiles.map((file, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-gray-50 rounded-lg border"
              >
                <div className="flex items-center space-x-3">
                  <div className="text-xl">
                    {file.name.endsWith('.xlsx') ? '📊' : 
                     file.name.endsWith('.csv') ? '📄' :
                     file.name.endsWith('.json') ? '📋' : '📄'}
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {file.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {formatFileSize(file.size)} • {templates[file.template]?.name}
                    </p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <span
                    className={`px-2 py-1 text-xs rounded-full font-medium ${
                      file.status === 'pending'
                        ? 'bg-yellow-100 text-yellow-800'
                        : file.status === 'processing'
                        ? 'bg-green-100 text-green-800'
                        : file.status === 'completed'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {file.status === 'pending' && 'Чака'}
                    {file.status === 'processing' && 'Обработва се'}
                    {file.status === 'completed' && 'Готов'}
                    {file.status === 'error' && 'Грешка'}
                  </span>
                  <button
                    onClick={() => removeFile(index)}
                    className="p-1 text-gray-400 hover:text-red-600 transition-colors"
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            ))}
          </div>
          
          <div className="flex justify-end space-x-3">
            <button
              onClick={() => setUploadedFiles([])}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
            >
              Изчисти всички
            </button>
            <button
              onClick={startImport}
              className="px-4 py-2 text-sm font-medium text-white bg-green-600 border border-transparent rounded-md hover:bg-green-700 transition-colors"
            >
              Започни универсален импорт
            </button>
          </div>
        </div>
      )}

      {/* Template Info */}
      <div className="bg-green-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-green-500 text-xl">💡</div>
          <div className="text-sm text-green-800">
            <div className="font-medium mb-1">За универсален импорт:</div>
            {selectedTemplate === 'custom' && (
              <ul className="space-y-1 text-sm">
                <li>• Свободно мапиране - можете да дефинирате как се свързват колоните</li>
                <li>• Поддръжка за комплексни структури от Excel и CSV</li>
                <li>• Валидация на данните преди окончателен импорт</li>
                <li>• Възможност за запазване на мапинга за бъдещи импорти</li>
              </ul>
            )}
            {selectedTemplate === 'journal' && (
              <ul className="space-y-1 text-sm">
                <li>• Стандартен формат: Дата, Документ, Описание, Сметка, Дебит, Кредит</li>
                <li>• Автоматична валидация за балансираност</li>
                <li>• Поддръжка за контрагенти и аналитика</li>
                <li>• Групиране на записи по документ</li>
              </ul>
            )}
            {selectedTemplate === 'accounts' && (
              <ul className="space-y-1 text-sm">
                <li>• Импорт на сметки от сметкоплана</li>
                <li>• Формат: Код, Наименование, Тип, Салдо</li>
                <li>• Автоматично създаване на йерархична структура</li>
                <li>• Валидация за дубликати и грешни кодове</li>
              </ul>
            )}
            {selectedTemplate === 'counterparts' && (
              <ul className="space-y-1 text-sm">
                <li>• Импорт на контрагенти с пълни данни</li>
                <li>• Формат: Име, БУЛСТАТ, ДДС номер, Адрес, Град</li>
                <li>• Автоматична валидация на БУЛСТАТ и ДДС</li>
                <li>• Обработка на дубликати</li>
              </ul>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}