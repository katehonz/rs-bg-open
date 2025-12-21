import { useState } from 'react';
import ControlisyImport from '../components/imports/ControlisyImport';
import BankImport from '../components/imports/BankImport';
import UniversalImport from '../components/imports/UniversalImport';
import UniversalExport from '../components/imports/UniversalExport';
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
      description: 'Импорт от JSON формат',
      acceptedFormats: ['.json'],
      features: [
        'JSON формат v2.0',
        'Сметкоплан и ДДС',
        'Валидация на данните',
        'Примерен шаблон'
      ]
    },
    export: {
      name: 'Универсален експорт',
      icon: '📤',
      description: 'Експорт на данни в JSON формат',
      acceptedFormats: ['.json'],
      features: [
        'Пълен или частичен експорт',
        'Съвместимост между инсталации',
        'JSON формат v2.0',
        'Експорт на активни записи'
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
      case 'export':
        return <UniversalExport />;
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
    </div>
  );
}