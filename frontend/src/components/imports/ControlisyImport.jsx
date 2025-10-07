import { useState, useCallback, useEffect } from 'react';
import ControlisyImportModal from './ControlisyImportModal';
import ControlisyReviewModal from './ControlisyReviewModal';

export default function ControlisyImport() {
  const [dragOver, setDragOver] = useState(false);
  const [uploadedFiles, setUploadedFiles] = useState([]);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importHistory, setImportHistory] = useState([]);
  const [selectedImport, setSelectedImport] = useState(null);
  const [showReviewModal, setShowReviewModal] = useState(false);
  const [reviewData, setReviewData] = useState(null);

  const acceptedFormats = ['.xml'];
  
  const features = [
    'Автоматично разпознаване на фактури',
    'Извличане на ДДС информация', 
    'Мапиране на контрагенти',
    'Банкови извлечения',
    'Универсален експорт на данни',
    'Поддръжка за покупки и продажби',
    'Валидация на балансираност'
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
        file: file
      }))]);
    }
  }, []);

  const handleFileSelect = useCallback((e) => {
    const files = Array.from(e.target.files);
    setUploadedFiles(prev => [...prev, ...files.map(file => ({
      name: file.name,
      size: file.size,
      type: file.type,
      status: 'pending',
      file: file
    }))]);
  }, []);

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

  const startImport = () => {
    if (uploadedFiles.length > 0) {
      setShowImportModal(true);
    }
  };

  const fetchImportHistory = async () => {
    try {
      const companyId = parseInt(localStorage.getItem('currentCompanyId')) || 1;
      // Use REST API for file operations
      const response = await fetch(`http://localhost:8080/api/controlisy/imports/${companyId}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (response.ok) {
        const importList = await response.json();
        console.log('Import list received:', importList);
        importList.forEach(imp => {
          console.log(`Import ${imp.id}: status="${imp.status}", file="${imp.file_name}"`);
        });
        setImportHistory(importList);
      } else {
        console.error('Failed to fetch import history:', response.status);
        setImportHistory([]); // Set empty list if no imports found
      }
    } catch (error) {
      console.error('Error fetching import history:', error);
      setImportHistory([]);
    }
  };

  const openReviewModal = async (importItem) => {
    try {
      // Use REST API to get import data
      const response = await fetch(`http://localhost:8080/api/controlisy/import/${importItem.id}`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (response.ok) {
        const importData = await response.json();
        setSelectedImport(importItem);
        setReviewData(importData.parsed_data);
        setShowReviewModal(true);
      } else {
        console.error('Failed to fetch import data:', response.status);
      }
    } catch (error) {
      console.error('Error loading staged data:', error);
    }
  };

  const processImport = async (importId) => {
    try {
      // Use REST API for file processing
      const response = await fetch(`http://localhost:8080/api/controlisy/process/${importId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (response.ok) {
        const result = await response.json();
        if (result.success) {
          alert(`Импортът е завършен успешно! Документите са създадени в счетоводните записи.`);
          fetchImportHistory(); // Refresh the list
          
          // Ask if user wants to view journal entries
          if (window.confirm('Искате ли да видите създадените счетоводни записи?')) {
            window.location.href = '/accounting/journal-entries';
          }
        }
      } else {
        const error = await response.json();
        alert(`Грешка при импорт: ${error.error || 'Неизвестна грешка'}`);
      }
    } catch (error) {
      console.error('Error processing import:', error);
      alert('Грешка при импорт: ' + error.message);
    }
  };

  const deleteImport = async (importId, fileName) => {
    if (!window.confirm(`Сигурни ли сте, че искате да изтриете импорта на "${fileName}"?`)) {
      return;
    }
    
    try {
      const response = await fetch(`http://localhost:8080/api/controlisy/imports/${importId}`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (response.ok) {
        fetchImportHistory(); // Refresh the list
      } else {
        alert('Грешка при изтриване на импорта');
      }
    } catch (error) {
      console.error('Error deleting import:', error);
      alert('Грешка при изтриване на импорта');
    }
  };

  const handleImportComplete = (results) => {
    console.log('Import completed:', results);
    setUploadedFiles([]);
    setShowImportModal(false);
    fetchImportHistory(); // Refresh import history
  };

  // Load import history on component mount
  useEffect(() => {
    fetchImportHistory();
  }, []);

  return (
    <div className="space-y-6">
      {/* Source Info */}
      <div className="flex items-start space-x-4">
        <div className="text-4xl">📄</div>
        <div>
          <h3 className="text-lg font-semibold text-gray-900">
            Controlisy импорт
          </h3>
          <p className="text-gray-600 mb-3">
            Импорт от PDF документи и XML файлове от Controlisy система за автоматично разпознаване на документи
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

      {/* Import History */}
      {importHistory.length > 0 && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-lg font-semibold text-gray-900">История на импортите</h4>
            <button
              onClick={fetchImportHistory}
              className="p-2 text-gray-500 hover:text-gray-700 rounded-full hover:bg-gray-100"
              title="Опресни"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
            <div className="max-h-96 overflow-y-auto">
              {importHistory.map((importItem) => (
                <div key={importItem.id} className="p-4 border-b border-gray-100 last:border-b-0">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-3">
                        <div className="text-lg">
                          {importItem.document_type === 'purchase' ? '🛒' : '💰'}
                        </div>
                        <div>
                          <div className="font-medium text-gray-900">{importItem.file_name}</div>
                          <div className="text-sm text-gray-500">
                            {new Date(importItem.import_date).toLocaleString('bg-BG')} • 
                            {importItem.imported_documents} док. • {importItem.imported_contractors} контрагенти
                          </div>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      <span className={`px-2 py-1 text-xs rounded-full font-medium ${
                        importItem.status === 'staged' 
                          ? 'bg-blue-100 text-blue-800'
                          : importItem.status === 'reviewed'
                          ? 'bg-yellow-100 text-yellow-800'
                          : importItem.status === 'processing'
                          ? 'bg-purple-100 text-purple-800'
                          : importItem.status === 'completed'
                          ? 'bg-green-100 text-green-800'
                          : 'bg-red-100 text-red-800'
                      }`}>
                        {importItem.status === 'staged' && 'За преглед'}
                        {importItem.status === 'reviewed' && 'Прегледан'}
                        {importItem.status === 'processing' && 'Обработва се'}
                        {importItem.status === 'completed' && 'Завършен'}
                        {importItem.status === 'failed' && 'Грешка'}
                      </span>
                      
                      {(importItem.status === 'staged' || importItem.status === 'reviewed') && (
                        <button
                          onClick={() => openReviewModal(importItem)}
                          className="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700"
                        >
                          Преглед
                        </button>
                      )}
                      
                      {(importItem.status === 'staged' || importItem.status === 'reviewed') && (
                        <button
                          onClick={() => processImport(importItem.id)}
                          className="px-3 py-1 text-xs bg-green-600 text-white rounded hover:bg-green-700"
                        >
                          Импортирай
                        </button>
                      )}
                      
                      <button
                        onClick={() => deleteImport(importItem.id, importItem.file_name)}
                        className="px-3 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700"
                        title="Изтрий импорт"
                      >
                        Изтрий
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Upload Area */}
      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragOver
            ? 'border-purple-500 bg-purple-50'
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="space-y-4">
          <div className="text-4xl">📁</div>
          <div>
            <p className="text-lg font-medium text-gray-900">
              Пуснете Controlisy файловете тук или
            </p>
            <label className="cursor-pointer">
              <span className="text-purple-600 hover:text-purple-700 font-medium">
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
            Максимум 10 файла, до 50MB всеки
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
                    {file.name.endsWith('.xml') ? '📋' : '📄'}
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {file.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {formatFileSize(file.size)} • {file.type || 'Unknown type'}
                    </p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <span
                    className={`px-2 py-1 text-xs rounded-full font-medium ${
                      file.status === 'pending'
                        ? 'bg-yellow-100 text-yellow-800'
                        : file.status === 'processing'
                        ? 'bg-purple-100 text-purple-800'
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
              className="px-4 py-2 text-sm font-medium text-white bg-purple-600 border border-transparent rounded-md hover:bg-purple-700 transition-colors"
            >
              Започни Controlisy импорт
            </button>
          </div>
        </div>
      )}

      {/* Info Section */}
      <div className="bg-purple-50 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <div className="text-purple-500 text-xl">💡</div>
          <div className="text-sm text-purple-800">
            <div className="font-medium mb-1">За Controlisy импорт:</div>
            <ul className="space-y-1 text-sm">
              <li>• XML файловете съдържат структурирани счетоводни данни</li>
              <li>• PDF файловете се обработват автоматично от Controlisy AI</li>
              <li>• Поддържат се както покупки (код 1), така и продажби (код 2)</li>
              <li>• Системата автоматично извлича ДДС информация и контрагенти</li>
              <li>• Аналитичните признаци (accountItem1-4) се игнорират</li>
            </ul>
          </div>
        </div>
      </div>

      {/* Import Modal */}
      <ControlisyImportModal
        show={showImportModal}
        onClose={() => setShowImportModal(false)}
        files={uploadedFiles}
        onImportComplete={handleImportComplete}
      />
      
      {/* Review Modal */}
      <ControlisyReviewModal
        show={showReviewModal}
        onClose={() => setShowReviewModal(false)}
        selectedImport={selectedImport}
        reviewData={reviewData}
        onDataUpdate={(updatedData) => setReviewData(updatedData)}
        onMarkReviewed={() => {
          fetchImportHistory(); // Refresh the list
          setShowReviewModal(false);
        }}
      />
    </div>
  );
}