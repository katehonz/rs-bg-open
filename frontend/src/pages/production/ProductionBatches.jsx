import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

export default function ProductionBatches() {
  const [batches, setBatches] = useState([]);
  const [technologyCards, setTechnologyCards] = useState([]);
  const [showModal, setShowModal] = useState(false);
  const [selectedBatch, setSelectedBatch] = useState(null);
  const [showStageExecution, setShowStageExecution] = useState(false);

  // Form state
  const [formData, setFormData] = useState({
    technologyCardId: '',
    batchNumber: '',
    inputQuantity: '',
    productionDate: new Date().toISOString().split('T')[0],
    notes: ''
  });

  useEffect(() => {
    // TODO: Fetch production batches from backend
    setBatches([]);

    // TODO: Fetch technology cards for dropdown
    setTechnologyCards([
      { id: 1, name: 'Производство на хляб', outputUnit: 'кг', stages: [] }
    ]);
  }, []);

  const handleCreate = () => {
    setFormData({
      technologyCardId: '',
      batchNumber: generateBatchNumber(),
      inputQuantity: '',
      productionDate: new Date().toISOString().split('T')[0],
      notes: ''
    });
    setShowModal(true);
  };

  const generateBatchNumber = () => {
    const date = new Date();
    return `BATCH-${date.getFullYear()}${String(date.getMonth() + 1).padStart(2, '0')}${String(date.getDate()).padStart(2, '0')}-${Date.now() % 10000}`;
  };

  const handleSave = async () => {
    // TODO: Implement GraphQL mutation
    console.log('Creating production batch:', formData);

    const newBatch = {
      id: Date.now(),
      ...formData,
      status: 'draft',
      createdAt: new Date().toISOString()
    };

    setBatches([newBatch, ...batches]);
    setShowModal(false);
  };

  const handleStartProduction = (batch) => {
    setSelectedBatch(batch);
    setShowStageExecution(true);
  };

  const getStatusLabel = (status) => {
    const statusLabels = {
      draft: 'Чернова',
      in_progress: 'В процес',
      completed: 'Завършена',
      cancelled: 'Отказана'
    };
    return statusLabels[status] || status;
  };

  const getStatusColor = (status) => {
    const statusColors = {
      draft: 'bg-gray-100 text-gray-800',
      in_progress: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-red-100 text-red-800'
    };
    return statusColors[status] || 'bg-gray-100 text-gray-800';
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
            className="border-blue-500 text-blue-600 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Производствени партиди
          </Link>
          <Link
            to="/production/reports"
            className="border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm"
          >
            Справки
          </Link>
        </nav>
      </div>

      {/* Section Header */}
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold text-gray-900">Производствени партиди</h2>
        <button
          onClick={handleCreate}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          + Нова партида
        </button>
      </div>

      {/* Batches List */}
      <div className="bg-white shadow-sm rounded-lg overflow-hidden">
        {batches.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <p className="mb-2">Няма създадени производствени партиди</p>
            <p className="text-sm">Създайте първата си производствена партида</p>
          </div>
        ) : (
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Номер партида
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Технологична карта
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Количество
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Дата
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Статус
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {batches.map((batch) => (
                <tr key={batch.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {batch.batchNumber}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {technologyCards.find(tc => tc.id === parseInt(batch.technologyCardId))?.name || '-'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {batch.inputQuantity} {technologyCards.find(tc => tc.id === parseInt(batch.technologyCardId))?.outputUnit || ''}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {new Date(batch.productionDate).toLocaleDateString('bg-BG')}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${getStatusColor(batch.status)}`}>
                      {getStatusLabel(batch.status)}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    {batch.status === 'draft' || batch.status === 'in_progress' ? (
                      <button
                        onClick={() => handleStartProduction(batch)}
                        className="text-blue-600 hover:text-blue-900"
                      >
                        {batch.status === 'draft' ? 'Започни' : 'Продължи'}
                      </button>
                    ) : (
                      <button className="text-gray-400 cursor-not-allowed">
                        Преглед
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Create Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-full max-w-2xl shadow-lg rounded-md bg-white">
            {/* Modal Header */}
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-gray-900">Нова производствена партида</h3>
              <button
                onClick={() => setShowModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Form */}
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Номер на партида *
                </label>
                <input
                  type="text"
                  value={formData.batchNumber}
                  onChange={(e) => setFormData({ ...formData, batchNumber: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Технологична карта *
                </label>
                <select
                  value={formData.technologyCardId}
                  onChange={(e) => setFormData({ ...formData, technologyCardId: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="">Изберете технологична карта</option>
                  {technologyCards.map(tc => (
                    <option key={tc.id} value={tc.id}>
                      {tc.name}
                    </option>
                  ))}
                </select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Входно количество *
                  </label>
                  <input
                    type="number"
                    step="0.0001"
                    value={formData.inputQuantity}
                    onChange={(e) => setFormData({ ...formData, inputQuantity: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    placeholder="0.00"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Дата на производство *
                  </label>
                  <input
                    type="date"
                    value={formData.productionDate}
                    onChange={(e) => setFormData({ ...formData, productionDate: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Бележки
                </label>
                <textarea
                  value={formData.notes}
                  onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Допълнителна информация"
                />
              </div>
            </div>

            {/* Modal Footer */}
            <div className="flex justify-end space-x-3 pt-4 mt-6 border-t">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
              >
                Отказ
              </button>
              <button
                onClick={handleSave}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Създай партида
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Stage Execution Modal */}
      {showStageExecution && selectedBatch && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-10 mx-auto p-5 border w-11/12 max-w-6xl shadow-lg rounded-md bg-white mb-10">
            {/* Modal Header */}
            <div className="flex justify-between items-center mb-4">
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  Изпълнение на партида {selectedBatch.batchNumber}
                </h3>
                <p className="text-sm text-gray-500 mt-1">
                  Технологична карта: {technologyCards.find(tc => tc.id === parseInt(selectedBatch.technologyCardId))?.name}
                </p>
              </div>
              <button
                onClick={() => setShowStageExecution(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Stages List */}
            <div className="space-y-4">
              <div className="p-4 bg-blue-50 rounded-lg">
                <p className="text-sm text-blue-800">
                  <strong>Входно количество:</strong> {selectedBatch.inputQuantity} {technologyCards.find(tc => tc.id === parseInt(selectedBatch.technologyCardId))?.outputUnit}
                </p>
              </div>

              {/* TODO: Display and execute stages */}
              <div className="text-center text-gray-500 py-8">
                <p>Етапи ще се изпълняват автоматично с създаване на счетоводни операции</p>
                <p className="text-sm mt-2">Тази функционалност изисква backend имплементация</p>
              </div>
            </div>

            {/* Modal Footer */}
            <div className="flex justify-end space-x-3 pt-4 mt-6 border-t">
              <button
                onClick={() => setShowStageExecution(false)}
                className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
              >
                Затвори
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
