import { useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { graphqlRequest } from '../utils/graphqlClient';

const DASHBOARD_QUERY = `
  query DashboardData($companyId: Int!, $fromDate: NaiveDate!, $toDate: NaiveDate!, $vatYear: Int!) {
    journalEntries(
      filter: { companyId: $companyId, fromDate: $fromDate, toDate: $toDate }
      limit: 50
    ) {
      id
      documentDate
      documentNumber
      description
      totalAmount
      totalVatAmount
      isPosted
      vatDocumentType
      entryNumber
      createdAt
    }
    vatReturns(filter: { companyId: $companyId, periodYear: $vatYear }, limit: 12) {
      id
      periodYear
      periodMonth
      status
      vatToPay
      vatToRefund
      dueDate
    }
    counterparts(companyId: $companyId) {
      id
      name
      city
      country
      isActive
      counterpartType
    }
    countEntryLines(companyId: $companyId, fromDate: $fromDate, toDate: $toDate, isPosted: true)
  }
`;

const statusLabels = {
  DRAFT: 'Чернова',
  SUBMITTED: 'Подадена',
  APPROVED: 'Одобрена',
};

const vatDocumentLabels = {
  '01': 'Фактура',
  '02': 'Дебитно известие',
  '03': 'Кредитно известие',
  '04': 'Протокол',
};

function formatCurrency(amount) {
  const value = Number(amount) || 0;
  return new Intl.NumberFormat('bg-BG', {
    style: 'currency',
    currency: 'BGN',
    maximumFractionDigits: 2,
  }).format(value);
}

function formatDate(date) {
  if (!date) return '-';
  const dt = new Date(date);
  return new Intl.DateTimeFormat('bg-BG').format(dt);
}

function getMonthLabel(month, year) {
  const names = [
    'Януари',
    'Февруари',
    'Март',
    'Април',
    'Май',
    'Юни',
    'Юли',
    'Август',
    'Септември',
    'Октомври',
    'Ноември',
    'Декември',
  ];
  const name = names[month - 1] || month;
  return `${name} ${year}`;
}

function SummaryCards({ metrics }) {
  const items = [
    {
      title: 'Оборот за текущия месец',
      value: formatCurrency(metrics.turnover),
      accent: 'bg-blue-100 text-blue-700',
      hint: `${metrics.totalEntries} записа • ${metrics.postedEntries} приключени`,
    },
    {
      title: 'Счетоводни редове (Дт/Кт)',
      value: metrics.entryLinesCount.toString(),
      accent: 'bg-purple-100 text-purple-700',
      hint: `${metrics.totalEntries} журнални записа`,
    },
    {
      title: 'Нетен ДДС за плащане',
      value: formatCurrency(metrics.vatNet),
      accent: metrics.vatNet > 0 ? 'bg-red-100 text-red-700' : 'bg-green-100 text-green-700',
      hint: `Начислен ДДС: ${formatCurrency(metrics.vatAccrued)}`,
    },
    {
      title: 'Чернови дневници',
      value: metrics.draftEntries.toString(),
      accent: metrics.draftEntries > 0 ? 'bg-amber-100 text-amber-700' : 'bg-green-100 text-green-700',
      hint:
        metrics.draftEntries > 0
          ? 'Има записи за приключване'
          : 'Всички записи са приключени',
    },
    {
      title: 'Активни контрагенти',
      value: metrics.activeCounterparts.toString(),
      accent: 'bg-teal-100 text-teal-700',
      hint: `${metrics.inactiveCounterparts} неактивни • ${metrics.citiesTracked} града`,
    },
  ];

  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-5">
      {items.map((item) => (
        <div
          key={item.title}
          className="bg-white border border-gray-100 shadow-sm rounded-lg px-5 py-4 flex flex-col justify-between"
        >
          <div>
            <p className="text-sm font-medium text-gray-500">{item.title}</p>
            <p className="mt-2 text-2xl font-semibold text-gray-900">{item.value}</p>
          </div>
          <p className={`mt-4 text-xs font-medium px-2 py-1 rounded-full inline-flex ${item.accent}`}>
            {item.hint}
          </p>
        </div>
      ))}
    </div>
  );
}

function RecentEntries({ entries }) {
  if (entries.length === 0) {
    return (
      <div className="bg-white border border-gray-100 shadow-sm rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Последни записвания</h3>
        <p className="text-sm text-gray-500">Няма записвания за текущия период.</p>
      </div>
    );
  }

  return (
    <div className="bg-white border border-gray-100 shadow-sm rounded-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">Последни записвания</h3>
        <Link to="/accounting/entries" className="text-sm font-medium text-blue-600 hover:text-blue-500">
          Виж всички →
        </Link>
      </div>
      <ul className="divide-y divide-gray-100">
        {entries.map((entry) => (
          <li key={entry.id} className="py-3 flex items-start justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <p className="text-sm font-medium text-gray-900">
                  {entry.documentNumber || entry.entryNumber || `Запис #${entry.id}`}
                </p>
                {entry.vatDocumentType && (
                  <span className="text-xs px-2 py-0.5 rounded-full bg-purple-100 text-purple-700">
                    {vatDocumentLabels[entry.vatDocumentType] || entry.vatDocumentType}
                  </span>
                )}
                {!entry.isPosted && (
                  <span className="text-xs px-2 py-0.5 rounded-full bg-amber-100 text-amber-700">
                    Чернова
                  </span>
                )}
              </div>
              <p className="mt-1 text-sm text-gray-600 line-clamp-2">{entry.description || 'Без описание'}</p>
              <p className="mt-1 text-xs text-gray-400">{formatDate(entry.documentDate)} • {formatDate(entry.createdAt)} (създаден)</p>
            </div>
            <div className="ml-4 text-right">
              <p className="text-sm font-semibold text-gray-900">{formatCurrency(entry.totalAmount)}</p>
              {entry.totalVatAmount ? (
                <p className="text-xs text-gray-500 mt-1">ДДС {formatCurrency(entry.totalVatAmount)}</p>
              ) : null}
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}

function VatCompliance({ upcoming, overdue }) {
  return (
    <div className="bg-white border border-gray-100 shadow-sm rounded-lg p-6 flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">ДДС календар</h3>
        <Link to="/vat/returns" className="text-sm font-medium text-blue-600 hover:text-blue-500">
          Управление →
        </Link>
      </div>

      <div className="space-y-5 flex-1">
        <section>
          <h4 className="text-sm font-semibold text-gray-700 mb-3">Предстоящи декларации</h4>
          {upcoming.length === 0 ? (
            <p className="text-sm text-gray-500">Няма предстоящи задължения.</p>
          ) : (
            <ul className="space-y-3">
              {upcoming.map((ret) => (
                <li key={ret.id} className="flex items-start justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {getMonthLabel(ret.periodMonth, ret.periodYear)}
                    </p>
                    <p className="text-xs text-gray-500">
                      Срок: {formatDate(ret.dueDate)} • Статус: {statusLabels[ret.status] || ret.status}
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-semibold text-gray-900">
                      {formatCurrency(Number(ret.vatToPay) - Number(ret.vatToRefund))}
                    </p>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section>
          <h4 className="text-sm font-semibold text-gray-700 mb-3">Просрочени</h4>
          {overdue.length === 0 ? (
            <p className="text-sm text-gray-500">Браво! Няма просрочени декларации.</p>
          ) : (
            <ul className="space-y-3">
              {overdue.map((ret) => (
                <li key={ret.id} className="flex items-start justify-between">
                  <div>
                    <p className="text-sm font-medium text-red-700">
                      {getMonthLabel(ret.periodMonth, ret.periodYear)}
                    </p>
                    <p className="text-xs text-red-500">
                      Крайният срок беше {formatDate(ret.dueDate)}
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-semibold text-red-700">
                      {formatCurrency(Number(ret.vatToPay) - Number(ret.vatToRefund))}
                    </p>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}

function CounterpartOverview({ snapshot }) {
  return (
    <div className="bg-white border border-gray-100 shadow-sm rounded-lg p-6 h-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">Контрагенти</h3>
        <Link to="/counterparts" className="text-sm font-medium text-blue-600 hover:text-blue-500">
          Управление →
        </Link>
      </div>
      <dl className="grid grid-cols-1 gap-y-4 text-sm text-gray-600">
        <div className="flex items-center justify-between">
          <dt className="font-medium text-gray-700">Активни</dt>
          <dd className="font-semibold text-gray-900">{snapshot.active}</dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="font-medium text-gray-700">Неактивни</dt>
          <dd className="text-gray-900">{snapshot.inactive}</dd>
        </div>
        <div className="flex items-center justify-between">
          <dt className="font-medium text-gray-700">Водещи градове</dt>
          <dd className="text-right text-gray-900">
            {snapshot.topCities.length === 0
              ? 'Няма данни'
              : snapshot.topCities
                  .map(([city, count]) => `${city} (${count})`)
                  .join(', ')}
          </dd>
        </div>
      </dl>
      <div className="mt-4 text-xs text-gray-500">
        Данните включват всички контрагенти на компанията.
      </div>
    </div>
  );
}

function TaskList({ tasks }) {
  return (
    <div className="bg-white border border-gray-100 shadow-sm rounded-lg p-6 h-full">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">Следващи действия</h3>
      </div>
      {tasks.length === 0 ? (
        <p className="text-sm text-gray-500">Няма активни задачи. Всичко е актуално.</p>
      ) : (
        <ul className="space-y-3">
          {tasks.map((task) => (
            <li key={task.title} className="p-3 rounded-md border border-gray-100 hover:bg-gray-50">
              <div className="flex items-start justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-900">{task.title}</p>
                  <p className="text-xs text-gray-500 mt-1">{task.description}</p>
                </div>
                {task.link && task.linkLabel && (
                  <Link to={task.link} className="text-xs font-medium text-blue-600 hover:text-blue-500">
                    {task.linkLabel}
                  </Link>
                )}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default function Dashboard() {
  const [companyId] = useState(parseInt(localStorage.getItem('currentCompanyId')) || 1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [journalEntries, setJournalEntries] = useState([]);
  const [vatReturns, setVatReturns] = useState([]);
  const [counterparts, setCounterparts] = useState([]);
  const [entryLinesCount, setEntryLinesCount] = useState(0);

  useEffect(() => {
    const fetchDashboard = async () => {
      setLoading(true);
      setError(null);

      const today = new Date();
      const startOfMonth = new Date(today.getFullYear(), today.getMonth(), 1);
      const startDate = startOfMonth.toISOString().split('T')[0];
      const endDate = today.toISOString().split('T')[0];
      const vatYear = today.getFullYear();

      try {
        const response = await graphqlRequest(DASHBOARD_QUERY, {
          companyId,
          fromDate: startDate,
          toDate: endDate,
          vatYear,
        });

        setJournalEntries(response.journalEntries || []);
        setVatReturns(response.vatReturns || []);
        setCounterparts(response.counterparts || []);
        setEntryLinesCount(response.countEntryLines || 0);
      } catch (err) {
        console.error('Error loading dashboard data:', err);
        setError(err.message || 'Възникна грешка при зареждане на данните.');
      } finally {
        setLoading(false);
      }
    };

    fetchDashboard();
  }, [companyId]);

  const metrics = useMemo(() => {
    const turnover = journalEntries.reduce((sum, entry) => sum + Number(entry.totalAmount || 0), 0);
    const vatAccrued = journalEntries.reduce((sum, entry) => sum + Number(entry.totalVatAmount || 0), 0);
    const totalEntries = journalEntries.length;
    const postedEntries = journalEntries.filter((entry) => entry.isPosted).length;
    const draftEntries = totalEntries - postedEntries;
    const activeCounterparts = counterparts.filter((cp) => cp.isActive).length;
    const inactiveCounterparts = counterparts.length - activeCounterparts;
    const citiesTracked = new Set(counterparts.filter((cp) => cp.city).map((cp) => cp.city)).size;
    const vatNet = vatReturns
      .filter((ret) => ret.status === 'DRAFT')
      .reduce((sum, ret) => sum + Number(ret.vatToPay || 0) - Number(ret.vatToRefund || 0), 0);

    const topCities = Object.entries(
      counterparts.reduce((acc, cp) => {
        const city = cp.city || 'Неуточнено';
        acc[city] = (acc[city] || 0) + 1;
        return acc;
      }, {})
    )
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3);

    return {
      turnover,
      vatAccrued,
      vatNet,
      totalEntries,
      postedEntries,
      draftEntries,
      activeCounterparts,
      inactiveCounterparts,
      citiesTracked,
      topCities,
      entryLinesCount,
    };
  }, [journalEntries, counterparts, vatReturns, entryLinesCount]);

  const recentEntries = useMemo(() => journalEntries.slice(0, 6), [journalEntries]);

  const { upcomingVat, overdueVat } = useMemo(() => {
    const today = new Date();
    const drafts = vatReturns.filter((ret) => ret.status === 'DRAFT');

    const overdue = drafts
      .filter((ret) => new Date(ret.dueDate) < today)
      .sort((a, b) => new Date(a.dueDate) - new Date(b.dueDate));

    const upcoming = drafts
      .filter((ret) => new Date(ret.dueDate) >= today)
      .sort((a, b) => new Date(a.dueDate) - new Date(b.dueDate))
      .slice(0, 5);

    return { upcomingVat: upcoming, overdueVat: overdue };
  }, [vatReturns]);

  const tasks = useMemo(() => {
    const list = [];

    if (metrics.draftEntries > 0) {
      list.push({
        title: 'Приключване на чернови записи',
        description: `${metrics.draftEntries} записи чакат осчетоводяване`,
        link: '/accounting/entries',
        linkLabel: 'Към дневника',
      });
    }

    if (overdueVat.length > 0) {
      list.push({
        title: 'Просрочени ДДС декларации',
        description: `${overdueVat.length} периода са с изтекъл срок`,
        link: '/vat/returns',
        linkLabel: 'Прегледай декларации',
      });
    }

    if (metrics.inactiveCounterparts > 0) {
      list.push({
        title: 'Преглед на неактивни контрагенти',
        description: `${metrics.inactiveCounterparts} контрагента са маркирани като неактивни`,
        link: '/counterparts',
        linkLabel: 'Управление',
      });
    }

    if (list.length === 0) {
      list.push({
        title: 'Всичко е под контрол',
        description: 'Няма задачи за момента. Продължавайте в този дух!',
      });
    }

    return list.slice(0, 3);
  }, [metrics.draftEntries, metrics.inactiveCounterparts, overdueVat]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Начално табло</h1>
          <p className="mt-1 text-sm text-gray-500">
            Обобщение на основните показатели за текущия месец.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Link
            to="/vat/returns"
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
          >
            Управление на ДДС
          </Link>
          <Link
            to="/accounting/entries"
            className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            Нов запис
          </Link>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
          ⚠️ {error}
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full" />
        </div>
      ) : (
        <>
          <SummaryCards metrics={metrics} />

          <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
            <div className="xl:col-span-2">
              <RecentEntries entries={recentEntries} />
            </div>
            <VatCompliance upcoming={upcomingVat} overdue={overdueVat} />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <CounterpartOverview
              snapshot={{
                active: metrics.activeCounterparts,
                inactive: metrics.inactiveCounterparts,
                topCities: metrics.topCities,
              }}
            />
            <TaskList tasks={tasks} />
          </div>
        </>
      )}
    </div>
  );
}
