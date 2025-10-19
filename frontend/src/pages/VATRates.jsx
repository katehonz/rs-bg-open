import { useState } from 'react';

// VAT Operation Codes - Reference Data from NAP specification
// Based on koloni-vat.pdf and vat_nastrojki.pdf from Unicontsoft ERP
const VAT_COLUMNS = [
  // Purchase diary (Дневник на покупките)
  { code: 'пок09', diary: 'Дневник на покупките', column: '09', description: 'Доставки, ВОП, доставки по чл.82, ал.2-4 и внос без право на ДК или без данък', active: true },
  { code: 'пок10', diary: 'Дневник на покупките', column: '10', description: 'Облагаеми доставки, ВОП, доставки по чл.82, ал.2-4, внос, доставки по чл.69, ал.2 с право на пълен ДК', active: true },
  { code: 'пок12', diary: 'Дневник на покупките', column: '12', description: 'Облагаеми доставки, ВОП, доставки по чл.82, ал.2-4, внос, доставки по чл.69, ал.2 с право на частичен ДК', active: true },
  { code: 'пок14', diary: 'Дневник на покупките', column: '14', description: 'Годишна корекция по чл.73, ал.8 ЗДДС (+/-) и чл.147, ал.3 ЗДДС', active: true },
  { code: 'пок15', diary: 'Дневник на покупките', column: '15', description: 'Придобиване на стоки от посредник в тристранна операция', active: true },

  // Sales diary (Дневник на продажбите)
  { code: 'про11', diary: 'Дневник на продажбите', column: '11', description: 'Облагаеми доставки със ставка 20%', active: true },
  { code: 'про12', diary: 'Дневник на продажбите', column: '12', description: 'Начислен данък (20%) предвиден в закона в други случаи', active: true },
  { code: 'про13', diary: 'Дневник на продажбите', column: '13', description: 'ВОП', active: true },
  { code: 'про14', diary: 'Дневник на продажбите', column: '14', description: 'Получени доставки по чл.82, ал.2-4 ЗДДС', active: true },
  { code: 'про16', diary: 'Дневник на продажбите', column: '16', description: 'Начислен данък за доставки на стоки и услуги за лични нужди', active: true },
  { code: 'про17', diary: 'Дневник на продажбите', column: '17', description: 'Облагаеми доставки със ставка 9%', active: true },
  { code: 'про19', diary: 'Дневник на продажбите', column: '19', description: 'Доставки със ставка 0% по глава 3-та от ЗДДС', active: true },
  { code: 'про20', diary: 'Дневник на продажбите', column: '20', description: 'ВОД (к3 VIES)', active: true },
  { code: 'про21', diary: 'Дневник на продажбите', column: '21', description: 'Доставки по чл.140, 146, 173 ЗДДС', active: true },
  { code: 'про22', diary: 'Дневник на продажбите', column: '22', description: 'Доставки на услуги по чл.21, ал.2 с място на изпълнение територията на друга страна членка (к5 VIES)', active: true },
  { code: 'про23-1', diary: 'Дневник на продажбите', column: '23-1', description: 'Доставки по чл.69, ал.2 и доставки при условията на дистанционни продажби с място на изпълнение територията на друга страна членка', active: true },
  { code: 'про23-2', diary: 'Дневник на продажбите', column: '23-2', description: 'Доставки (дан. основа и ДДС) при условията на дистанционни продажби с място на изпълнение територията на друга страна членка', active: true },
  { code: 'про24-1', diary: 'Дневник на продажбите', column: '24-1', description: 'Освободени доставки в страната и освободени ВОП', active: true },
  { code: 'про24-2', diary: 'Дневник на продажбите', column: '24-2', description: 'Освободени ВОД (к3 VIES)', active: true },
  { code: 'про24-3', diary: 'Дневник на продажбите', column: '24-3', description: 'Освободени доставки на услуги по чл.21, ал.2 с място на изпълнение територията на друга страна членка (к5 VIES)', active: true },
  { code: 'про25', diary: 'Дневник на продажбите', column: '25', description: 'Доставки като посредник в тристранни операции (к4 VIES)', active: true },

  // Legacy codes (старата система)
  { code: '1', diary: 'Дневник на покупките', column: '1', description: 'Сделки и внос с право на данъчен кредит', active: false },
  { code: '2', diary: 'Дневник на покупките', column: '2', description: 'Сделки и внос с право на частичен данъчен кредит', active: false },
  { code: '3', diary: 'Дневник на покупките', column: '3', description: 'Сделки с нерегистрирани лица, освободени, или без право на ДК', active: false },
  { code: '4', diary: 'Дневник на продажбите', column: '4', description: 'Облагаеми сделки', active: false },
  { code: '5', diary: 'Дневник на продажбите', column: '5', description: 'Освободени сделки', active: false },
  { code: '6', diary: 'Дневник на продажбите', column: '6', description: 'Сделки за износ', active: false },
];

// Detailed VAT settings - Based on vat_nastrojki.pdf
const VAT_OPERATIONS = [
  // Purchase operations
  { code: '1-09-1', basis: 'Без право на ДК или без данък: Доставки и внос', purchaseColumn: 'пок09', salesColumn: '', printName: 'Без право на ДК или без данък: Доставки и внос', vatRate: 0, type: 'Покупка', country: '', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-09-2', basis: 'Без право на ДК или без данък: ВОП и доставки по чл.82, ал.2-4 ЗДДС', purchaseColumn: 'пок09', salesColumn: '24-1', printName: 'Без право на ДК или без данък: ВОП и доставки по чл.82, ал.2-4 ЗДДС', vatRate: 0, type: 'Покупка', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-1', basis: 'С право на пълен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2', purchaseColumn: 'пок10', salesColumn: '', printName: 'С право на пълен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2', vatRate: 20, type: 'Покупка', country: 'Б', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-2', basis: 'С право на пълен ДК: ВОП', purchaseColumn: 'пок10', salesColumn: '13', printName: 'С право на пълен ДК: ВОП', vatRate: 20, type: 'Покупка', country: '', taxBase: 'K', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-3', basis: 'С право на пълен ДК: Доставки по чл.82, ал.2-4', purchaseColumn: 'пок10', salesColumn: '14', printName: 'С право на пълен ДК: Доставки по чл.82, ал.2-4', vatRate: 20, type: 'Покупка', country: '', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-4', basis: 'С право на пълен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2 (9%)', purchaseColumn: 'пок10', salesColumn: '', printName: 'С право на пълен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2 (9%)', vatRate: 9, type: 'Покупка', country: 'Г', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-5', basis: 'С право на пълен ДК: Обратно начисляване по чл.163а, ал.2 от ЗДДС (01)', purchaseColumn: 'пок10', salesColumn: '14', printName: 'С право на пълен ДК: Обратно начисляване по чл.163а, ал.2 от ЗДДС (01)', vatRate: 20, type: 'Покупка', country: '01', taxBase: 'AE', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-10-6', basis: 'С право на пълен ДК: Обратно начисляване по чл.163а, ал.2 от ЗДДС (02)', purchaseColumn: 'пок10', salesColumn: '14', printName: 'С право на пълен ДК: Обратно начисляване по чл.163а, ал.2 от ЗДДС (02)', vatRate: 20, type: 'Покупка', country: '02', taxBase: 'AE', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-12-1', basis: 'С право на частичен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2', purchaseColumn: 'пок12', salesColumn: '', printName: 'С право на частичен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2', vatRate: 20, type: 'Покупка', country: 'Б', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-12-2', basis: 'С право на частичен ДК: ВОП', purchaseColumn: 'пок12', salesColumn: '13', printName: 'С право на частичен ДК: ВОП', vatRate: 20, type: 'Покупка', country: '', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-12-3', basis: 'С право на частичен ДК: Доставки по чл.82, ал.2-4', purchaseColumn: 'пок12', salesColumn: '14', printName: 'С право на частичен ДК: Доставки по чл.82, ал.2-4', vatRate: 20, type: 'Покупка', country: '', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-12-4', basis: 'С право на частичен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2 (9%)', purchaseColumn: 'пок12', salesColumn: '', printName: 'С право на частичен ДК: Облагаеми доставки, внос, доставки по чл.69, ал.2 (9%)', vatRate: 9, type: 'Покупка', country: 'Г', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '1-15', basis: 'Придобиване на стоки от посредник в тристранна операция', purchaseColumn: 'пок15', salesColumn: '', printName: 'Придобиване на стоки от посредник в тристранна операция', vatRate: 0, type: 'Покупка', country: '', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },

  // Sales operations
  { code: '2-11', basis: 'Облагаеми доставки със ставка 20%', purchaseColumn: '', salesColumn: '11', printName: '', vatRate: 20, type: 'Продажба', country: 'Б', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-11-1', basis: 'Обратно начисляване по чл.163а, ал.2 от ЗДДС (01)', purchaseColumn: '', salesColumn: '11', printName: 'Обратно начисляване по чл.163а, ал.2 от ЗДДС, част I на Приложение 2', vatRate: 0, type: 'Продажба', country: '01', taxBase: 'AE', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-11-2', basis: 'Обратно начисляване по чл.163а, ал.2 от ЗДДС (02)', purchaseColumn: '', salesColumn: '11', printName: 'Обратно начисляване по чл.163а, ал.2 от ЗДДС, част II на Приложение 2', vatRate: 0, type: 'Продажба', country: '02', taxBase: 'AE', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-17', basis: 'Облагаеми доставки със ставка 9%', purchaseColumn: '', salesColumn: '17', printName: '', vatRate: 9, type: 'Продажба', country: 'Г', taxBase: 'S', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-19-1', basis: 'Облагаеми доставки със ставка 0%', purchaseColumn: '', salesColumn: '19', printName: '', vatRate: 0, type: 'Продажба', country: 'А', taxBase: 'Z', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-19-2', basis: 'Доставки със ставка 0% по глава 3-та от ЗДДС', purchaseColumn: '', salesColumn: '19', printName: 'Доставки със ставка 0% по глава 3-та от ЗДДС', vatRate: 0, type: 'Продажба', country: '', taxBase: 'Z', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-20', basis: 'ВОД (к3 VIES)', purchaseColumn: '', salesColumn: '20', printName: 'ВОД;  Обратно начисляване', vatRate: 0, type: 'Продажба', country: '', taxBase: 'K', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-21-1', basis: 'Доставки по чл.140, 146, 173, ал.1 и 4 от ЗДДС', purchaseColumn: '', salesColumn: '21', printName: 'Доставки по чл.140, 146, 173, ал.1 и 4 от ЗДДС', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-21-2', basis: 'Доставка по чл.163а, ал.2 от ЗДДС', purchaseColumn: '', salesColumn: '21', printName: 'Доставка по чл.163а, ал.2 от ЗДДС', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-22', basis: 'Доставки на услуги по чл.21, ал.2 с място на изпълнение територията на друга страна членка (к5 VIES)', purchaseColumn: '', salesColumn: '22', printName: 'Доставки на услуги по чл.21, ал.2;  Обратно начисляване', vatRate: 0, type: 'Продажба', country: '', taxBase: 'K', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-23', basis: 'Доставки по чл.69, ал.2 и доставки при условията на дистанционни продажби с място на изпълнение територията на друга страна членка', purchaseColumn: '', salesColumn: '23-1', printName: 'Доставки по чл.69, ал.2 и доставки при условията на дистанционни продажби с място на изпълнение', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-24-1', basis: 'Освободени доставки в страната и освободени ВОП', purchaseColumn: '', salesColumn: '24-1', printName: 'Освободени доставки и освободени ВОП', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-24-2', basis: 'Освободени ВОД (к3 VIES)', purchaseColumn: '', salesColumn: '24-2', printName: 'Освободени ВОД', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-24-3', basis: 'Освободени доставки на услуги по чл.21, ал.2 с място на изпълнение територията на друга страна членка (к5 VIES)', purchaseColumn: '', salesColumn: '24-3', printName: 'Освободени доставки на услуги по чл.21, ал.2', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-25', basis: 'Доставки като посредник в тристранни операции (к4 VIES)', purchaseColumn: '', salesColumn: '25', printName: '141 2006/112/ЕО', vatRate: 0, type: 'Продажба', country: '', taxBase: 'E', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-99', basis: 'Неначисляване на ДДС по чл.113, ал.9', purchaseColumn: '', salesColumn: '', printName: 'Неначисляване на ДДС по чл.113, ал.9 от ЗДДС', vatRate: 0, type: 'Продажба', country: '', taxBase: 'O', taxCredit: '', intrastat: '', source: '', active: true },

  // Service codes
  { code: '1-xx', basis: 'Не участва в дневник на покупките', purchaseColumn: '', salesColumn: '', printName: '', vatRate: 0, type: 'Покупка', country: 'Б', taxBase: '', taxCredit: '', intrastat: '', source: '', active: true },
  { code: '2-xx', basis: 'Не участва в дневник на продажбите', purchaseColumn: '', salesColumn: '', printName: '', vatRate: 0, type: 'Продажба', country: 'Б', taxCredit: '', intrastat: '', source: '', active: true },
];

export default function VATRates() {
  const [activeTab, setActiveTab] = useState('columns'); // columns, operations
  const [searchQuery, setSearchQuery] = useState('');
  const [filterDiary, setFilterDiary] = useState('all'); // all, purchases, sales
  const [showOnlyActive, setShowOnlyActive] = useState(true);

  // Filter columns based on search and diary filter
  const filteredColumns = VAT_COLUMNS.filter(col => {
    const matchesSearch = searchQuery === '' ||
      col.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
      col.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      col.column.includes(searchQuery);

    const matchesDiary = filterDiary === 'all' ||
      (filterDiary === 'purchases' && col.diary === 'Дневник на покупките') ||
      (filterDiary === 'sales' && col.diary === 'Дневник на продажбите');

    const matchesActive = !showOnlyActive || col.active;

    return matchesSearch && matchesDiary && matchesActive;
  });

  // Filter operations
  const filteredOperations = VAT_OPERATIONS.filter(op => {
    const matchesSearch = searchQuery === '' ||
      op.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
      op.basis.toLowerCase().includes(searchQuery.toLowerCase()) ||
      op.purchaseColumn.includes(searchQuery) ||
      op.salesColumn.includes(searchQuery);

    const matchesType = filterDiary === 'all' ||
      (filterDiary === 'purchases' && op.type === 'Покупка') ||
      (filterDiary === 'sales' && op.type === 'Продажба');

    const matchesActive = !showOnlyActive || op.active;

    return matchesSearch && matchesType && matchesActive;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">ДДС Настройки и Ставки</h1>
          <p className="mt-1 text-sm text-gray-500">
            Референтни номенклатури за ДДС операции (само четене, програмно дефинирани)
          </p>
        </div>
        <div className="flex items-center space-x-2 text-xs text-gray-500">
          <span className="px-2 py-1 bg-blue-50 text-blue-700 rounded">📚 NAP 2025</span>
          <span className="px-2 py-1 bg-green-50 text-green-700 rounded">🔒 Read-only</span>
        </div>
      </div>

      {/* Info Box */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <div className="flex items-start">
          <div className="text-blue-400 mr-3 text-xl">ℹ️</div>
          <div className="text-sm text-blue-800">
            <p className="font-medium mb-1">Тези настройки са програмно дефинирани според NAP спецификация</p>
            <p className="text-blue-700">
              Кодовете и операциите са взети от референтна търговска ERP система (10+ години на пазара).
              Промените се правят само програмно при актуализация на законодателството.
            </p>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { key: 'columns', label: 'Колони в дневниците', icon: '📋', count: filteredColumns.length },
            { key: 'operations', label: 'Детайлни операции', icon: '⚙️', count: filteredOperations.length }
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center space-x-2 ${
                activeTab === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
              <span className="px-2 py-0.5 bg-gray-100 text-gray-600 rounded-full text-xs">{tab.count}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Filters */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Търсене
            </label>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Търси по код, описание или колона..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Дневник
            </label>
            <select
              value={filterDiary}
              onChange={(e) => setFilterDiary(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="all">Всички</option>
              <option value="purchases">Покупки</option>
              <option value="sales">Продажби</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Статус
            </label>
            <div className="flex items-center h-10">
              <input
                type="checkbox"
                id="showOnlyActive"
                checked={showOnlyActive}
                onChange={(e) => setShowOnlyActive(e.target.checked)}
                className="h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <label htmlFor="showOnlyActive" className="ml-2 text-sm text-gray-700">
                Само активни
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Columns Tab */}
      {activeTab === 'columns' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Колони на дневниците</h3>
            <p className="text-sm text-gray-500">Кодове за колони в Дневник на покупките и Дневник на продажбите</p>
          </div>

          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                    Код
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-48">
                    Дневник
                  </th>
                  <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                    Колона
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Описание на колоната
                  </th>
                  <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                    Активен
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {filteredColumns.map((col, index) => (
                  <tr key={index} className={!col.active ? 'bg-gray-50 opacity-60' : ''}>
                    <td className="px-4 py-3 text-sm font-mono font-semibold text-blue-600">
                      {col.code}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-700">
                      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                        col.diary === 'Дневник на покупките'
                          ? 'bg-purple-100 text-purple-800'
                          : 'bg-green-100 text-green-800'
                      }`}>
                        {col.diary === 'Дневник на покупките' ? '📥' : '📤'} {col.diary}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-sm text-center font-mono font-bold text-gray-900">
                      {col.column}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-600">
                      {col.description}
                    </td>
                    <td className="px-4 py-3 text-center">
                      {col.active ? (
                        <span className="text-green-600">✅</span>
                      ) : (
                        <span className="text-gray-400">❌</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {filteredColumns.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                Няма намерени записи с текущите филтри
              </div>
            )}
          </div>
        </div>
      )}

      {/* Operations Tab */}
      {activeTab === 'operations' && (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900">Детайлни ДДС операции</h3>
            <p className="text-sm text-gray-500">Пълни настройки за всяка ДДС операция</p>
          </div>

          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                    Код
                  </th>
                  <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Основание за прилагане
                  </th>
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                    Покупки
                  </th>
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                    Продажби
                  </th>
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-20">
                    ДДС %
                  </th>
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-24">
                    Тип
                  </th>
                  <th className="px-3 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider w-16">
                    Визуал.
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {filteredOperations.map((op, index) => (
                  <tr key={index} className={!op.active ? 'bg-gray-50 opacity-60' : ''}>
                    <td className="px-3 py-3 text-sm font-mono font-semibold text-blue-600">
                      {op.code}
                    </td>
                    <td className="px-3 py-3 text-sm text-gray-700">
                      {op.basis}
                      {op.printName && op.printName !== op.basis && (
                        <div className="text-xs text-gray-500 mt-1">
                          Печат: {op.printName}
                        </div>
                      )}
                    </td>
                    <td className="px-3 py-3 text-sm text-center font-mono font-medium text-purple-600">
                      {op.purchaseColumn || '-'}
                    </td>
                    <td className="px-3 py-3 text-sm text-center font-mono font-medium text-green-600">
                      {op.salesColumn || '-'}
                    </td>
                    <td className="px-3 py-3 text-sm text-center font-semibold">
                      {op.vatRate > 0 ? `${op.vatRate}%` : '-'}
                    </td>
                    <td className="px-3 py-3 text-center">
                      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                        op.type === 'Покупка'
                          ? 'bg-purple-100 text-purple-800'
                          : 'bg-green-100 text-green-800'
                      }`}>
                        {op.type === 'Покупка' ? '📥' : '📤'}
                      </span>
                    </td>
                    <td className="px-3 py-3 text-sm text-center font-mono text-gray-500">
                      {op.taxBase || '-'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {filteredOperations.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                Няма намерени записи с текущите филтри
              </div>
            )}
          </div>
        </div>
      )}

      {/* Legend */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h4 className="text-sm font-semibold text-gray-800 mb-3">Легенда:</h4>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
          <div>
            <span className="font-medium text-gray-700">Визуализация кодове:</span>
            <ul className="mt-2 space-y-1 text-gray-600">
              <li><code className="px-1 bg-gray-200 rounded">S</code> - Стандартна ставка</li>
              <li><code className="px-1 bg-gray-200 rounded">K</code> - Комбинирана</li>
              <li><code className="px-1 bg-gray-200 rounded">E</code> - Освободена</li>
              <li><code className="px-1 bg-gray-200 rounded">Z</code> - Нулева ставка</li>
            </ul>
          </div>
          <div>
            <span className="font-medium text-gray-700">Кодове (продължение):</span>
            <ul className="mt-2 space-y-1 text-gray-600">
              <li><code className="px-1 bg-gray-200 rounded">AE</code> - Обратно начисляване</li>
              <li><code className="px-1 bg-gray-200 rounded">O</code> - Неначисляване</li>
              <li><code className="px-1 bg-gray-200 rounded">А</code> - Износ</li>
              <li><code className="px-1 bg-gray-200 rounded">Б/Г</code> - Държава</li>
            </ul>
          </div>
          <div>
            <span className="font-medium text-gray-700">Документация:</span>
            <ul className="mt-2 space-y-1 text-gray-600">
              <li>📄 Базирано на <strong>koloni-vat.pdf</strong></li>
              <li>⚙️ Детайли от <strong>vat_nastrojki.pdf</strong></li>
              <li>📚 NAP спецификация PPDDS_2025</li>
              <li>🏢 Unicontsoft Dreem Personal 1.5</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
