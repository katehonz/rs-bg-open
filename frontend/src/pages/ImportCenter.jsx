import { useState } from 'react';
import ControlisyImport from '../components/imports/ControlisyImport';
import BankImport from '../components/imports/BankImport';
import UniversalImport from '../components/imports/UniversalImport';
import AIInvoiceScanner from '../components/imports/AIInvoiceScanner';

export default function ImportCenter() {
  const [activeTab, setActiveTab] = useState('ai');

  const importSources = {
    ai: {
      name: 'AI Сканиране',
      icon: '🤖',
      description: 'AI сканиране на фактури с Mistral Vision',
      acceptedFormats: ['.png', '.jpg', '.jpeg', '.pdf'],
      features: [
        'Mistral AI Vision',
        'Автоматично извличане на данни',
        'VIES валидация на ДДС',
        'PDF и изображения'
      ]
    },
    controlisy: {
      name: 'Controlisy',
      icon: '📄',
      description: 'Импорт от XML файлове от Controlisy с универсален експорт',
      acceptedFormats: ['.xml'],
      features: [
        'Автоматично разпознаване на фактури',
        'Извличане на ДДС информация',
        'Мапиране на контрагенти',
        'Универсален експорт на данни'
      ]
    },
    bank: {
      name: 'Банкови извлечения',
      icon: '🏦',
      description: 'Импорт на банкови извлечения и операции',
      acceptedFormats: ['.xml', '.csv', '.txt'],
      features: [
        'MT940 формат',
        'CSV файлове от банки',
        'Автоматично мапиране на сметки',
        'Разпознаване на плащания'
      ]
    },
    universal: {
      name: 'Универсален импорт',
      icon: '📊',
      description: 'Импорт от Excel, CSV и други формати',
      acceptedFormats: ['.xlsx', '.csv', '.json', '.xml'],
      features: [
        'Свободен мапинг на полета',
        'Bulk импорт на записи',
        'Валидация и предварителен преглед',
        'Шаблони за импорт'
      ]
    }
  };

  const renderImportComponent = () => {
    switch (activeTab) {
      case 'ai':
        return <AIInvoiceScanner />;
      case 'controlisy':
        return <ControlisyImport />;
      case 'bank':
        return <BankImport />;
      case 'universal':
        return <UniversalImport />;
      default:
        return <AIInvoiceScanner />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Център за импорти</h1>
        <p className="mt-2 text-gray-600">
          Импорт на счетоводни данни от различни източници
        </p>
      </div>

      {/* Source Selection Tabs */}
      <div className="bg-white shadow rounded-lg">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6">
            {Object.entries(importSources).map(([key, source]) => (
              <button
                key={key}
                onClick={() => setActiveTab(key)}
                className={`py-4 px-1 border-b-2 font-medium text-sm flex items-center space-x-2 ${
                  activeTab === key
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <span className="text-lg">{source.icon}</span>
                <span>{source.name}</span>
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {/* Render the appropriate import component */}
          {renderImportComponent()}
        </div>
      </div>

      {/* Import History */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">История на импортите</h3>
        </div>
        <div className="divide-y divide-gray-200">
          {/* Example import history items */}
          <div className="px-6 py-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-900">
                  Controlisy импорт - 15 документа
                </p>
                <p className="text-sm text-gray-500">
                  Днес в 14:30 • 2 грешки, 13 успешни
                </p>
              </div>
              <div className="flex items-center space-x-2">
                <span className="px-2 py-1 text-xs bg-yellow-100 text-yellow-800 rounded-full">
                  Изисква внимание
                </span>
                <button className="text-blue-600 hover:text-blue-700 text-sm">
                  Преглед
                </button>
              </div>
            </div>
          </div>
          <div className="px-6 py-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-900">
                  Банкови извлечения - 28 операции
                </p>
                <p className="text-sm text-gray-500">
                  Вчера в 09:15 • Всички успешни
                </p>
              </div>
              <div className="flex items-center space-x-2">
                <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">
                  Завършен
                </span>
                <button className="text-blue-600 hover:text-blue-700 text-sm">
                  Преглед
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="text-2xl">📊</div>
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Общо импорти</p>
              <p className="text-2xl font-bold text-gray-900">47</p>
            </div>
          </div>
        </div>
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="text-2xl">✅</div>
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">Успешни</p>
              <p className="text-2xl font-bold text-green-600">43</p>
            </div>
          </div>
        </div>
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="text-2xl">⚠️</div>
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-600">С грешки</p>
              <p className="text-2xl font-bold text-yellow-600">4</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}