export { default as IntrastatDashboard } from './IntrastatDashboardSimple';

// Простите компоненти за демонстрация
export const IntrastatReports = () => (
  <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
    <h2 className="text-xl font-bold mb-4">📊 INTRASTAT Справки</h2>
    <div className="text-gray-600">
      <p>Тук ще се показват детайлни справки за:</p>
      <ul className="list-disc list-inside mt-2 space-y-1">
        <li>Входящи операции по страни и CN кодове</li>
        <li>Изходящи операции по страни и CN кодове</li>
        <li>Анализ на прагове по месеци</li>
        <li>Експорт в Excel/CSV формат</li>
      </ul>
      <div className="mt-4 p-4 bg-blue-50 rounded-lg">
        <p className="text-blue-800 font-medium">💡 В разработка</p>
        <p className="text-blue-700 text-sm">Модулът ще бъде завършен с пълната функционалност.</p>
      </div>
    </div>
  </div>
);

export const IntrastatSettings = () => (
  <div className="bg-white rounded-lg shadow border border-gray-200 p-6">
    <h2 className="text-xl font-bold mb-4">⚙️ INTRASTAT Настройки</h2>
    <div className="text-gray-600">
      <p>Тук ще се конфигурират:</p>
      <ul className="list-disc list-inside mt-2 space-y-1">
        <li>Активиране/деактивиране на модула</li>
        <li>Задаване на прагове за входящи/изходящи</li>
        <li>Свързване на сметки с INTRASTAT номенклатура</li>
        <li>Импорт на CN кодове от CSV</li>
        <li>Настройки за отговорното лице</li>
      </ul>
      <div className="mt-4 p-4 bg-green-50 rounded-lg">
        <p className="text-green-800 font-medium">✅ Backend готов</p>
        <p className="text-green-700 text-sm">API и база данни са напълно имплементирани.</p>
      </div>
    </div>
  </div>
);
