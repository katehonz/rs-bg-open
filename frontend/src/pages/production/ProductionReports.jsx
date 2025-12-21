import { useState } from 'react';
import { Link } from 'react-router-dom';

export default function ProductionReports() {
  const [reportType, setReportType] = useState('batches');
  const [filters, setFilters] = useState({
    dateFrom: '',
    dateTo: '',
    technologyCardId: '',
    status: ''
  });

  const handleGenerateReport = () => {
    // TODO: Implement report generation
    console.log('Generating report:', reportType, filters);
  };

  return (
    <div className="container mx-auto px-4 py-6">
      {/* Page Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Производство</h1>
        <p className="text-sm text-gray-600">
          Производствено счетоводство с технологични карти и автоматични операции
        </p>
      </div>

      {/* Tabs Navigation */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <Link
            to="/production/technology-cards"
            className="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Технологични карти
          </Link>
          <Link
            to="/production/batches"
            className="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Производствени партиди
          </Link>
          <Link
            to="/production/reports"
            className="border-blue-500 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Справки
          </Link>
        </nav>
      </div>

      {/* Section Header */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-2">Производствени справки</h2>
        <p className="text-sm text-gray-600">
          Анализи и отчети за производствена дейност
        </p>
      </div>

      {/* Report Type Selection */}
      <div className="bg-white shadow-sm rounded-lg p-6 mb-6">
        <h3 className="text-sm font-medium text-gray-700 mb-3">Тип справка</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <button
            onClick={() => setReportType('batches')}
            className={`p-4 border-2 rounded-lg text-left transition-colors ${
              reportType === 'batches'
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <div className="font-medium text-gray-900">Производствени партиди</div>
            <div className="text-sm text-gray-500 mt-1">
              Справка за изпълнени производствени партиди по период
            </div>
          </button>

          <button
            onClick={() => setReportType('technology-cards')}
            className={`p-4 border-2 rounded-lg text-left transition-colors ${
              reportType === 'technology-cards'
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <div className="font-medium text-gray-900">Производство по карти</div>
            <div className="text-sm text-gray-500 mt-1">
              Обобщена справка по технологични карти
            </div>
          </button>

          <button
            onClick={() => setReportType('accounts')}
            className={`p-4 border-2 rounded-lg text-left transition-colors ${
              reportType === 'accounts'
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <div className="font-medium text-gray-900">Счетоводно отражение</div>
            <div className="text-sm text-gray-500 mt-1">
              Операции по сметки от производството
            </div>
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="bg-white shadow-sm rounded-lg p-6 mb-6">
        <h3 className="text-sm font-medium text-gray-700 mb-3">Филтри</h3>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              От дата
            </label>
            <input
              type="date"
              value={filters.dateFrom}
              onChange={(e) => setFilters({ ...filters, dateFrom: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              До дата
            </label>
            <input
              type="date"
              value={filters.dateTo}
              onChange={(e) => setFilters({ ...filters, dateTo: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          {reportType === 'batches' && (
            <>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Технологична карта
                </label>
                <select
                  value={filters.technologyCardId}
                  onChange={(e) => setFilters({ ...filters, technologyCardId: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="">Всички</option>
                  {/* TODO: Load technology cards */}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Статус
                </label>
                <select
                  value={filters.status}
                  onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="">Всички</option>
                  <option value="draft">Чернова</option>
                  <option value="in_progress">В процес</option>
                  <option value="completed">Завършена</option>
                  <option value="cancelled">Отказана</option>
                </select>
              </div>
            </>
          )}
        </div>

        <div className="mt-4 flex justify-end">
          <button
            onClick={handleGenerateReport}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            Генерирай справка
          </button>
        </div>
      </div>

      {/* Report Results */}
      <div className="bg-white shadow-sm rounded-lg p-6">
        <h3 className="text-sm font-medium text-gray-700 mb-4">Резултати</h3>

        {reportType === 'batches' && (
          <div className="text-center text-gray-500 py-12">
            <svg className="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p className="text-sm font-medium">Справка за производствени партиди</p>
            <p className="text-xs mt-2">Изберете период и натиснете "Генерирай справка"</p>
          </div>
        )}

        {reportType === 'technology-cards' && (
          <div className="text-center text-gray-500 py-12">
            <svg className="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
            </svg>
            <p className="text-sm font-medium">Справка по технологични карти</p>
            <p className="text-xs mt-2">Обобщено производство по всяка технологична карта</p>
          </div>
        )}

        {reportType === 'accounts' && (
          <div className="text-center text-gray-500 py-12">
            <svg className="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
            </svg>
            <p className="text-sm font-medium">Счетоводни операции от производство</p>
            <p className="text-xs mt-2">Автоматично създадени операции по етапи</p>
          </div>
        )}

        {/* TODO: Display actual report data */}
        <div className="mt-6 p-4 bg-yellow-50 border border-yellow-200 rounded-md">
          <p className="text-sm text-yellow-800">
            Справките изискват backend имплементация с GraphQL заявки за обобщение на данни
          </p>
        </div>
      </div>
    </div>
  );
}
