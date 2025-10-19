import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';
import AdminUserCompanies from '../components/AdminUserCompanies';
import AIAccountingSettings from '../components/Settings/AIAccountingSettings';

// User Periods Modal Component
function PeriodsModal({ user, onClose, onSave }) {
  const [periods, setPeriods] = useState({
    documentPeriodStart: user.documentPeriodStart,
    documentPeriodEnd: user.documentPeriodEnd,
    documentPeriodActive: user.documentPeriodActive,
    accountingPeriodStart: user.accountingPeriodStart,
    accountingPeriodEnd: user.accountingPeriodEnd,
    accountingPeriodActive: user.accountingPeriodActive,
    vatPeriodStart: user.vatPeriodStart,
    vatPeriodEnd: user.vatPeriodEnd,
    vatPeriodActive: user.vatPeriodActive
  });
  const [saving, setSaving] = useState(false);

  const UPDATE_PERIODS_MUTATION = `
    mutation UpdateUserInputPeriods($id: Int!, $input: UpdateInputPeriodsInput!) {
      updateUserInputPeriods(id: $id, input: $input) {
        id
      }
    }
  `;

  const handleSave = async () => {
    try {
      setSaving(true);
      await graphqlRequest(UPDATE_PERIODS_MUTATION, { 
        id: user.id, 
        input: periods 
      });
      alert('Периодите са обновени успешно!');
      onSave();
      onClose();
    } catch (err) {
      alert('Грешка при запазване: ' + err.message);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">
              Лични периоди - {user.firstName} {user.lastName}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <div className="space-y-6">
            {/* Document Period */}
            <div className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-md font-medium text-gray-900">Документен период</h4>
                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={periods.documentPeriodActive}
                    onChange={(e) => setPeriods({...periods, documentPeriodActive: e.target.checked})}
                    className="form-checkbox h-4 w-4 text-blue-600"
                  />
                  <span className="ml-2 text-sm text-gray-700">Активен</span>
                </label>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">От дата</label>
                  <input
                    type="date"
                    value={periods.documentPeriodStart}
                    onChange={(e) => setPeriods({...periods, documentPeriodStart: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">До дата</label>
                  <input
                    type="date"
                    value={periods.documentPeriodEnd}
                    onChange={(e) => setPeriods({...periods, documentPeriodEnd: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
              </div>
            </div>

            {/* Accounting Period */}
            <div className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-md font-medium text-gray-900">Счетоводен период</h4>
                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={periods.accountingPeriodActive}
                    onChange={(e) => setPeriods({...periods, accountingPeriodActive: e.target.checked})}
                    className="form-checkbox h-4 w-4 text-blue-600"
                  />
                  <span className="ml-2 text-sm text-gray-700">Активен</span>
                </label>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">От дата</label>
                  <input
                    type="date"
                    value={periods.accountingPeriodStart}
                    onChange={(e) => setPeriods({...periods, accountingPeriodStart: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">До дата</label>
                  <input
                    type="date"
                    value={periods.accountingPeriodEnd}
                    onChange={(e) => setPeriods({...periods, accountingPeriodEnd: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
              </div>
            </div>

            {/* VAT Period */}
            <div className="border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-md font-medium text-gray-900">ДДС период</h4>
                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={periods.vatPeriodActive}
                    onChange={(e) => setPeriods({...periods, vatPeriodActive: e.target.checked})}
                    className="form-checkbox h-4 w-4 text-blue-600"
                  />
                  <span className="ml-2 text-sm text-gray-700">Активен</span>
                </label>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">От дата</label>
                  <input
                    type="date"
                    value={periods.vatPeriodStart}
                    onChange={(e) => setPeriods({...periods, vatPeriodStart: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">До дата</label>
                  <input
                    type="date"
                    value={periods.vatPeriodEnd}
                    onChange={(e) => setPeriods({...periods, vatPeriodEnd: e.target.value})}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  />
                </div>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-end space-x-3 mt-6">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
            >
              Отказ
            </button>
            <button
              onClick={handleSave}
              disabled={saving}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              {saving ? 'Запазва...' : 'Запази'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// Create User Modal Component
function CreateUserModal({ onClose, onSave }) {
  const [formData, setFormData] = useState({
    username: '',
    email: '',
    password: '',
    firstName: '',
    lastName: '',
    groupId: 3 // Default to 'Счетоводител'
  });
  const [creating, setCreating] = useState(false);

  const CREATE_USER_MUTATION = `
    mutation CreateUser($input: CreateUserInput!) {
      createUser(input: $input) {
        id
      }
    }
  `;

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      setCreating(true);
      await graphqlRequest(CREATE_USER_MUTATION, { input: formData });
      alert('Потребителят е създаден успешно!');
      onSave();
      onClose();
    } catch (err) {
      alert('Грешка при създаване: ' + err.message);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-1/2 shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">Нов потребител</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Име</label>
                <input
                  type="text"
                  required
                  value={formData.firstName}
                  onChange={(e) => setFormData({...formData, firstName: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Фамилия</label>
                <input
                  type="text"
                  required
                  value={formData.lastName}
                  onChange={(e) => setFormData({...formData, lastName: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Потребителско име</label>
              <input
                type="text"
                required
                value={formData.username}
                onChange={(e) => setFormData({...formData, username: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Email</label>
              <input
                type="email"
                required
                value={formData.email}
                onChange={(e) => setFormData({...formData, email: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Парола</label>
              <input
                type="password"
                required
                minLength="6"
                value={formData.password}
                onChange={(e) => setFormData({...formData, password: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Роля</label>
              <select
                value={formData.groupId}
                onChange={(e) => setFormData({...formData, groupId: parseInt(e.target.value)})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              >
                <option value={1}>Суперадмин</option>
                <option value={2}>Админ</option>
                <option value={3}>Счетоводител</option>
              </select>
            </div>

            <div className="flex items-center justify-end space-x-3 mt-6">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
              >
                Отказ
              </button>
              <button
                type="submit"
                disabled={creating}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
              >
                {creating ? 'Създава...' : 'Създай'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

// Edit User Modal Component
function EditUserModal({ user, onClose, onSave }) {
  const [formData, setFormData] = useState({
    username: user.username,
    email: user.email,
    firstName: user.firstName,
    lastName: user.lastName,
    groupId: user.groupId,
    isActive: user.isActive
  });
  const [updating, setUpdating] = useState(false);

  const UPDATE_USER_MUTATION = `
    mutation UpdateUser($id: Int!, $input: UpdateUserInput!) {
      updateUser(id: $id, input: $input) {
        id
      }
    }
  `;

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      setUpdating(true);
      await graphqlRequest(UPDATE_USER_MUTATION, { 
        id: user.id, 
        input: formData 
      });
      alert('Потребителят е обновен успешно!');
      onSave();
      onClose();
    } catch (err) {
      alert('Грешка при обновяване: ' + err.message);
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-1/2 shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">Редактирай потребител</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Име</label>
                <input
                  type="text"
                  required
                  value={formData.firstName}
                  onChange={(e) => setFormData({...formData, firstName: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Фамилия</label>
                <input
                  type="text"
                  required
                  value={formData.lastName}
                  onChange={(e) => setFormData({...formData, lastName: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Потребителско име</label>
              <input
                type="text"
                required
                value={formData.username}
                onChange={(e) => setFormData({...formData, username: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Email</label>
              <input
                type="email"
                required
                value={formData.email}
                onChange={(e) => setFormData({...formData, email: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Роля</label>
              <select
                value={formData.groupId}
                onChange={(e) => setFormData({...formData, groupId: parseInt(e.target.value)})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              >
                <option value={1}>Суперадмин</option>
                <option value={2}>Админ</option>
                <option value={3}>Счетоводител</option>
              </select>
            </div>

            <div>
              <label className="inline-flex items-center">
                <input
                  type="checkbox"
                  checked={formData.isActive}
                  onChange={(e) => setFormData({...formData, isActive: e.target.checked})}
                  className="form-checkbox h-4 w-4 text-blue-600"
                />
                <span className="ml-2 text-sm text-gray-700">Потребителят е активен</span>
              </label>
            </div>

            <div className="flex items-center justify-end space-x-3 mt-6">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
              >
                Отказ
              </button>
              <button
                type="submit"
                disabled={updating}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
              >
                {updating ? 'Обновява...' : 'Обнови'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

// Users Tab Component
function UsersPage() {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showPeriodsModal, setShowPeriodsModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState(null);

  const USERS_QUERY = `
    query GetUsers {
      users {
        id
        username
        email
        firstName
        lastName
        groupId
        isActive
        documentPeriodStart
        documentPeriodEnd
        documentPeriodActive
        accountingPeriodStart
        accountingPeriodEnd
        accountingPeriodActive
        vatPeriodStart
        vatPeriodEnd
        vatPeriodActive
        createdAt
      }
    }
  `;

  const DELETE_USER_MUTATION = `
    mutation DeleteUser($id: Int!) {
      deleteUser(id: $id)
    }
  `;

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      setLoading(true);
      const response = await graphqlRequest(USERS_QUERY);
      setUsers(response.users || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (userId) => {
    if (!confirm('Сигурни ли сте, че искате да изтриете този потребител?')) {
      return;
    }

    try {
      await graphqlRequest(DELETE_USER_MUTATION, { id: userId });
      await loadUsers();
      alert('Потребителят е изтрит успешно!');
    } catch (err) {
      alert('Грешка при изтриването: ' + err.message);
    }
  };

  const handleEditPeriods = (user) => {
    setSelectedUser(user);
    setShowPeriodsModal(true);
  };

  const handleEditUser = (user) => {
    setSelectedUser(user);
    setShowEditModal(true);
  };

  const getInitials = (user) => {
    const name = `${user.firstName} ${user.lastName}`;
    return name.split(' ').map(word => word.charAt(0)).join('');
  };

  const getFullName = (user) => {
    return `${user.firstName} ${user.lastName}`;
  };

  const getRoleDisplay = (groupId) => {
    const roles = {
      1: 'Суперадмин',
      2: 'Админ', 
      3: 'Счетоводител'
    };
    return roles[groupId] || 'Потребител';
  };

  const getRoleColor = (groupId) => {
    const colors = {
      1: 'bg-purple-100 text-purple-800',
      2: 'bg-blue-100 text-blue-800',
      3: 'bg-green-100 text-green-800'
    };
    return colors[groupId] || 'bg-gray-100 text-gray-800';
  };

  const getPeriodsStatus = (user) => {
    const activeCount = [
      user.documentPeriodActive,
      user.accountingPeriodActive, 
      user.vatPeriodActive
    ].filter(Boolean).length;
    
    if (activeCount === 3) return { text: 'Всички активни', color: 'text-green-600' };
    if (activeCount > 0) return { text: `${activeCount}/3 активни`, color: 'text-yellow-600' };
    return { text: 'Няма активни', color: 'text-red-600' };
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        <span className="ml-2 text-gray-600">Зарежда се...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="text-red-800">Грешка при зареждане: {error}</div>
        <button 
          onClick={loadUsers}
          className="mt-2 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Опитай отново
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Потребители</h1>
          <p className="mt-1 text-sm text-gray-500">
            Управление на потребители и лични периоди
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <button 
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors"
          >
            <svg className="-ml-1 mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4"></path>
            </svg>
            Нов потребител
          </button>
        </div>
      </div>

      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        {users.length === 0 ? (
          <div className="p-6 text-center">
            <div className="text-gray-500">
              <div className="text-4xl mb-4">👥</div>
              <p>Няма намерени потребители.</p>
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Потребител
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Email
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Потребителско име
                  </th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Статус
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Роля
                  </th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Лични периоди
                  </th>
                  <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Действия
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {users.map((user) => (
                  <tr key={user.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="w-10 h-10 bg-gradient-to-r from-green-400 to-blue-500 rounded-full flex items-center justify-center">
                          <span className="text-white font-medium text-sm">
                            {getInitials(user)}
                          </span>
                        </div>
                        <div className="ml-4">
                          <div className="text-sm font-medium text-gray-900">
                            {getFullName(user)}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {user.email}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {user.username}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-center">
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                        user.isActive ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      }`}>
                        {user.isActive ? 'Активен' : 'Неактивен'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${getRoleColor(user.groupId)}`}>
                        {getRoleDisplay(user.groupId)}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-center">
                      <div className="flex flex-col items-center">
                        <button 
                          onClick={() => handleEditPeriods(user)}
                          className="text-blue-600 hover:text-blue-900 text-xs"
                        >
                          Настрой периоди
                        </button>
                        <div className={`text-xs mt-1 ${getPeriodsStatus(user).color}`}>
                          {getPeriodsStatus(user).text}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-center text-sm font-medium">
                      <div className="flex items-center justify-center space-x-2">
                        <button 
                          onClick={() => handleEditUser(user)}
                          className="text-blue-600 hover:text-blue-900"
                        >
                          Редактирай
                        </button>
                        <button 
                          onClick={() => handleDelete(user.id)}
                          className="text-red-600 hover:text-red-900"
                        >
                          Изтрий
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Modals */}
      {showPeriodsModal && (
        <PeriodsModal 
          user={selectedUser}
          onClose={() => setShowPeriodsModal(false)}
          onSave={loadUsers}
        />
      )}

      {showCreateModal && (
        <CreateUserModal 
          onClose={() => setShowCreateModal(false)}
          onSave={loadUsers}
        />
      )}

      {showEditModal && (
        <EditUserModal 
          user={selectedUser}
          onClose={() => setShowEditModal(false)}
          onSave={loadUsers}
        />
      )}
    </div>
  );
}

// Edit Company Modal Component
function EditCompanyModal({ company, onClose, onSave }) {
  const [formData, setFormData] = useState({
    name: company.name,
    eik: company.eik,
    vatNumber: company.vatNumber || '',
    address: company.address || '',
    city: company.city || '',
    country: company.country || '',
    phone: company.phone || '',
    email: company.email || '',
    contactPerson: company.contactPerson || '',
    managerName: company.managerName || '',
    authorizedPerson: company.authorizedPerson || '',
    managerEgn: company.managerEgn || '',
    authorizedPersonEgn: company.authorizedPersonEgn || '',
    isActive: company.isActive
  });
  const [updating, setUpdating] = useState(false);

  const UPDATE_COMPANY_MUTATION = `
    mutation UpdateCompany($id: Int!, $input: UpdateCompanyInput!) {
      updateCompany(id: $id, input: $input) {
        id
        name
        eik
        vatNumber
        city
        address
        country
        phone
        email
        contactPerson
        managerName
        authorizedPerson
        managerEgn
        authorizedPersonEgn
        isActive
      }
    }
  `;

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      setUpdating(true);
      await graphqlRequest(UPDATE_COMPANY_MUTATION, { 
        id: company.id, 
        input: {
          name: formData.name,
          eik: formData.eik,
          vatNumber: formData.vatNumber || null,
          address: formData.address || null,
          city: formData.city || null,
          country: formData.country || null,
          phone: formData.phone || null,
          email: formData.email || null,
          contactPerson: formData.contactPerson || null,
          managerName: formData.managerName || null,
          authorizedPerson: formData.authorizedPerson || null,
          managerEgn: formData.managerEgn || null,
          authorizedPersonEgn: formData.authorizedPersonEgn || null,
          isActive: formData.isActive
        }
      });
      alert('Фирмата е обновена успешно!');
      onSave();
      onClose();
    } catch (err) {
      alert('Грешка при обновяване: ' + err.message);
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-2/3 lg:w-1/2 shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">Редактирай фирма</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Име на фирмата</label>
                <input
                  type="text"
                  required
                  value={formData.name}
                  onChange={(e) => setFormData({...formData, name: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">ЕИК</label>
                <input
                  type="text"
                  required
                  value={formData.eik}
                  onChange={(e) => setFormData({...formData, eik: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">ДДС номер</label>
              <input
                type="text"
                value={formData.vatNumber}
                onChange={(e) => setFormData({...formData, vatNumber: e.target.value})}
                placeholder="BG123456789"
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Адрес</label>
              <input
                type="text"
                value={formData.address}
                onChange={(e) => setFormData({...formData, address: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Град</label>
                <input
                  type="text"
                  value={formData.city}
                  onChange={(e) => setFormData({...formData, city: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Държава</label>
                <input
                  type="text"
                  value={formData.country}
                  onChange={(e) => setFormData({...formData, country: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Телефон</label>
                <input
                  type="text"
                  value={formData.phone}
                  onChange={(e) => setFormData({...formData, phone: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Email</label>
                <input
                  type="email"
                  value={formData.email}
                  onChange={(e) => setFormData({...formData, email: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700">Лице за контакт</label>
              <input
                type="text"
                value={formData.contactPerson}
                onChange={(e) => setFormData({...formData, contactPerson: e.target.value})}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Управител</label>
                <input
                  type="text"
                  value={formData.managerName}
                  onChange={(e) => setFormData({...formData, managerName: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">ЕГН на управител</label>
                <input
                  type="text"
                  value={formData.managerEgn}
                  onChange={(e) => setFormData({...formData, managerEgn: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Упълномощено лице</label>
                <input
                  type="text"
                  value={formData.authorizedPerson}
                  onChange={(e) => setFormData({...formData, authorizedPerson: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">ЕГН на упълномощено лице</label>
                <input
                  type="text"
                  value={formData.authorizedPersonEgn}
                  onChange={(e) => setFormData({...formData, authorizedPersonEgn: e.target.value})}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                />
              </div>
            </div>

            <div>
              <label className="inline-flex items-center">
                <input
                  type="checkbox"
                  checked={formData.isActive}
                  onChange={(e) => setFormData({...formData, isActive: e.target.checked})}
                  className="form-checkbox h-4 w-4 text-blue-600"
                />
                <span className="ml-2 text-sm text-gray-700">Фирмата е активна</span>
              </label>
            </div>

            <div className="flex items-center justify-end space-x-3 mt-6">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
              >
                Отказ
              </button>
              <button
                type="submit"
                disabled={updating}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
              >
                {updating ? 'Обновява...' : 'Обнови'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

// Companies Tab Component  
function CompaniesPage() {
  const [companies, setCompanies] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedCompany, setSelectedCompany] = useState(null);
  const [currentCompanyId, setCurrentCompanyId] = useState(
    parseInt(localStorage.getItem('currentCompanyId')) || 1
  );

  const COMPANIES_QUERY = `
    query GetCompanies {
      companies {
        id
        name
        eik
        vatNumber
        address
        city
        country
        phone
        email
        contactPerson
        managerName
        authorizedPerson
        managerEgn
        authorizedPersonEgn
        isActive
      }
    }
  `;

  useEffect(() => {
    fetchCompanies();
  }, []);

  const fetchCompanies = async () => {
    try {
      setLoading(true);
      const response = await graphqlRequest(COMPANIES_QUERY);
      setCompanies(response.companies);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const setActiveCompany = async (companyId) => {
    try {
      setCurrentCompanyId(companyId);
      localStorage.setItem('currentCompanyId', companyId.toString());
      // Trigger a page reload or update context to refresh all components
      window.location.reload();
    } catch (err) {
      setError(err.message);
    }
  };

  const handleEditCompany = (company) => {
    setSelectedCompany(company);
    setShowEditModal(true);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full"></div>
        <span className="ml-2 text-gray-600">Зарежда се...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="text-red-800">Грешка при зареждане: {error}</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Фирми</h1>
          <p className="mt-1 text-sm text-gray-500">
            Управление на фирми в системата
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <button className="inline-flex items-center px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors">
            <svg className="-ml-1 mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 4v16m8-8H4"></path>
            </svg>
            Нова фирма
          </button>
        </div>
      </div>

      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Фирма
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ЕИК
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Град
                </th>
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Статус
                </th>
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Активна
                </th>
                <th className="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Действия
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {companies.map((company, index) => (
                <tr key={index} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="w-10 h-10 bg-gradient-to-r from-blue-400 to-blue-500 rounded-lg flex items-center justify-center">
                        <span className="text-white font-bold text-sm">
                          {company.name.charAt(0)}
                        </span>
                      </div>
                      <div className="ml-4">
                        <div className="text-sm font-medium text-gray-900">
                          {company.name}
                        </div>
                        {currentCompanyId === company.id && (
                          <div className="text-xs text-green-600 flex items-center">
                            <div className="w-1.5 h-1.5 bg-green-500 rounded-full mr-1"></div>
                            Текуща фирма
                          </div>
                        )}
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {company.eik}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {company.city}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-center">
                    <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                      company.isActive ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                    }`}>
                      {company.isActive ? 'Активна' : 'Неактивна'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-center">
                    {currentCompanyId === company.id ? (
                      <div className="flex items-center justify-center space-x-2">
                        <div className="w-4 h-4 bg-green-500 rounded-full"></div>
                        <span className="text-xs text-green-600">Текуща</span>
                      </div>
                    ) : (
                      <button 
                        onClick={() => setActiveCompany(company.id)}
                        className="text-sm text-blue-600 hover:text-blue-700 px-2 py-1 rounded hover:bg-blue-50"
                      >
                        Направи активна
                      </button>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-center text-sm font-medium">
                    <div className="flex items-center justify-center space-x-2">
                      <button 
                        onClick={() => handleEditCompany(company)}
                        className="text-blue-600 hover:text-blue-900"
                      >
                        Редактирай
                      </button>
                      {currentCompanyId !== company.id && (
                        <button className="text-red-600 hover:text-red-900">
                          Изтрий
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Edit Company Modal */}
      {showEditModal && (
        <EditCompanyModal 
          company={selectedCompany}
          onClose={() => setShowEditModal(false)}
          onSave={fetchCompanies}
        />
      )}
    </div>
  );
}

// System Settings Tab Component
function SystemPage() {
  const [companies, setCompanies] = useState([]);
  const [selectedCompanyId, setSelectedCompanyId] = useState(null);
  const [enableViesValidation, setEnableViesValidation] = useState(false);
  const [enableAiMapping, setEnableAiMapping] = useState(false);
  const [autoValidateOnImport, setAutoValidateOnImport] = useState(false);
  const [loadingCompanies, setLoadingCompanies] = useState(true);
  const [saving, setSaving] = useState(false);

  const [mistralApiKey, setMistralApiKey] = useState('');
  const [savingMistralKey, setSavingMistralKey] = useState(false);
  const [loadingContragentSettings, setLoadingContragentSettings] = useState(true);
  const [contragentSummary, setContragentSummary] = useState(null);
  const [loadingSummary, setLoadingSummary] = useState(false);
  const [processingGlobalContragents, setProcessingGlobalContragents] = useState(false);

  const [databaseStatus, setDatabaseStatus] = useState(null);
  const [loadingDatabaseStatus, setLoadingDatabaseStatus] = useState(true);
  const [backupInProgress, setBackupInProgress] = useState(false);
  const [optimizeInProgress, setOptimizeInProgress] = useState(false);
  const [lastOptimizationResult, setLastOptimizationResult] = useState(null);
  const [restoreInProgress, setRestoreInProgress] = useState(null);
  const [objectStorageSettings, setObjectStorageSettings] = useState({
    enabled: false,
    endpoint: '',
    accessKey: '',
    region: 'eu-central',
    bucket: '',
    prefix: '',
    forcePathStyle: true,
    hasSecretKey: false,
    secretKey: '',
    secretProvided: false
  });
  const [loadingObjectStorage, setLoadingObjectStorage] = useState(true);
  const [savingObjectStorage, setSavingObjectStorage] = useState(false);

  const COMPANIES_QUERY = `
    query {
      companies {
        id
        name
        enableViesValidation
        enableAiMapping
        autoValidateOnImport
      }
    }
  `;

  const UPDATE_COMPANY_SETTINGS_MUTATION = `
    mutation UpdateCompanyIntegration($input: UpdateCompanyIntegrationSettingsInput!) {
      updateCompanyIntegrationSettings(input: $input) {
        id
        enableViesValidation
        enableAiMapping
        autoValidateOnImport
      }
    }
  `;

  const CONTRAGENT_SETTINGS_QUERY = `
    query {
      contragentSettings {
        key
        value
        description
        encrypted
        updatedAt
      }
    }
  `;

  const UPSERT_CONTRAGENT_SETTING_MUTATION = `
    mutation UpsertContragentSetting($input: UpsertContragentSettingInput!) {
      upsertContragentSetting(input: $input) {
        id
        key
        value
        encrypted
      }
    }
  `;

  const PROCESS_GLOBAL_CONTRAGENTS_MUTATION = `
    mutation {
      processGlobalContragents {
        processed
        failed
      }
    }
  `;

  const GLOBAL_CONTRAGENT_SUMMARY_QUERY = `
    query {
      globalContragentSummary {
        total
        validCount
        invalidCount
        lastSyncedAt
      }
    }
  `;

  const DATABASE_STATUS_QUERY = `
    query {
      databaseMaintenanceStatus {
        databaseName
        databaseSizeBytes
        databaseSizePretty
        backupsDirectory
        objectStorageEnabled
        recentBackups {
          fileName
          fullPath
          sizeBytes
          sizePretty
          createdAt
          storageType
          objectKey
        }
      }
    }
  `;

  const OBJECT_STORAGE_SETTINGS_QUERY = `
    query {
      objectStorageSettings {
        enabled
        endpoint
        accessKey
        hasSecretKey
        region
        bucket
        prefix
        forcePathStyle
      }
    }
  `;

  const CREATE_BACKUP_MUTATION = `
    mutation {
      createDatabaseBackup {
        backup {
          fileName
          fullPath
          sizePretty
          createdAt
          storageType
          objectKey
        }
        durationMs
        remoteObjectKey
      }
    }
  `;

  const OPTIMIZE_DATABASE_MUTATION = `
    mutation {
      optimizeDatabase {
        databaseName
        sizeBeforePretty
        sizeAfterPretty
        sizeBeforeBytes
        sizeAfterBytes
        vacuumRan
        analyzeRan
        reindexRan
        durationMs
      }
    }
  `;

  const RESTORE_DATABASE_MUTATION = `
    mutation RestoreDatabase($input: RestoreDatabaseInput!) {
      restoreDatabase(input: $input) {
        source
        storageType
        durationMs
        startedAt
      }
    }
  `;

  const UPDATE_OBJECT_STORAGE_SETTINGS_MUTATION = `
    mutation UpdateObjectStorageSettings($input: UpdateObjectStorageSettingsInput!) {
      updateObjectStorageSettings(input: $input) {
        enabled
        endpoint
        accessKey
        hasSecretKey
        region
        bucket
        prefix
        forcePathStyle
      }
    }
  `;

  useEffect(() => {
    loadCompanies();
    loadContragentSettings();
    loadContragentSummary();
    loadDatabaseStatus();
    loadObjectStorageSettings();
  }, []);

  const loadCompanies = async () => {
    try {
      setLoadingCompanies(true);
      const response = await graphqlRequest(COMPANIES_QUERY);
      setCompanies(response.companies || []);

      if (response.companies && response.companies.length > 0) {
        const firstCompany = response.companies[0];
        setSelectedCompanyId(firstCompany.id);
        setEnableViesValidation(firstCompany.enableViesValidation || false);
        setEnableAiMapping(firstCompany.enableAiMapping || false);
        setAutoValidateOnImport(firstCompany.autoValidateOnImport || false);
      }
    } catch (err) {
      alert('Грешка при зареждане на компаниите: ' + err.message);
    } finally {
      setLoadingCompanies(false);
    }
  };

  const loadContragentSettings = async () => {
    try {
      setLoadingContragentSettings(true);
      const response = await graphqlRequest(CONTRAGENT_SETTINGS_QUERY);
      const settings = response.contragentSettings || [];
      const mistralSetting = settings.find((setting) => setting.key === 'mistral.api.key');
      setMistralApiKey(mistralSetting?.value || '');
    } catch (err) {
      console.error('Грешка при зареждане на AI настройките:', err);
      alert('Грешка при зареждане на AI настройките: ' + err.message);
    } finally {
      setLoadingContragentSettings(false);
    }
  };

  const loadContragentSummary = async () => {
    try {
      setLoadingSummary(true);
      const response = await graphqlRequest(GLOBAL_CONTRAGENT_SUMMARY_QUERY);
      setContragentSummary(response.globalContragentSummary || null);
    } catch (err) {
      console.error('Грешка при зареждане на статистиката за контрагентите:', err);
    } finally {
      setLoadingSummary(false);
    }
  };

  const loadDatabaseStatus = async () => {
    try {
      setLoadingDatabaseStatus(true);
      const response = await graphqlRequest(DATABASE_STATUS_QUERY);
      setDatabaseStatus(response.databaseMaintenanceStatus || null);
    } catch (err) {
      console.error('Грешка при зареждане на състоянието на базата данни:', err);
      alert('Грешка при зареждане на информацията за базата данни: ' + err.message);
    } finally {
      setLoadingDatabaseStatus(false);
    }
  };

  const loadObjectStorageSettings = async () => {
    try {
      setLoadingObjectStorage(true);
      const response = await graphqlRequest(OBJECT_STORAGE_SETTINGS_QUERY);
      const settings = response.objectStorageSettings;
      if (settings) {
        setObjectStorageSettings({
          enabled: settings.enabled,
          endpoint: settings.endpoint || '',
          accessKey: settings.accessKey || '',
          region: settings.region || 'eu-central',
          bucket: settings.bucket || '',
          prefix: settings.prefix || '',
          forcePathStyle: settings.forcePathStyle ?? true,
          hasSecretKey: settings.hasSecretKey || false,
          secretKey: '',
          secretProvided: false
        });
      } else {
        setObjectStorageSettings((prevSettings) => ({
          ...prevSettings,
          enabled: false,
          endpoint: '',
          accessKey: '',
          region: 'eu-central',
          bucket: '',
          prefix: '',
          forcePathStyle: true,
          hasSecretKey: false,
          secretKey: '',
          secretProvided: false
        }));
      }
    } catch (err) {
      console.error('Грешка при зареждане на Object Storage настройките:', err);
      alert('Грешка при зареждане на Object Storage настройките: ' + err.message);
    } finally {
      setLoadingObjectStorage(false);
    }
  };

  const handleCompanyChange = (companyId) => {
    const company = companies.find(c => c.id === parseInt(companyId, 10));
    if (company) {
      setSelectedCompanyId(company.id);
      setEnableViesValidation(company.enableViesValidation || false);
      setEnableAiMapping(company.enableAiMapping || false);
      setAutoValidateOnImport(company.autoValidateOnImport || false);
    }
  };

  const handleSaveApiSettings = async () => {
    if (!selectedCompanyId) {
      alert('Моля, изберете компания');
      return;
    }

    try {
      setSaving(true);
      await graphqlRequest(UPDATE_COMPANY_SETTINGS_MUTATION, {
        input: {
          companyId: selectedCompanyId,
          enableViesValidation,
          enableAiMapping,
          autoValidateOnImport
        }
      });

      localStorage.setItem('enableViesValidation', enableViesValidation);
      localStorage.setItem('enableAiMapping', enableAiMapping);
      localStorage.setItem('autoValidateOnImport', autoValidateOnImport);

      alert('Настройките за контрагентите са запазени успешно!');
      await loadCompanies(); // Reload to get updated data
    } catch (err) {
      alert('Грешка при запазване: ' + err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleSaveMistralSettings = async () => {
    try {
      setSavingMistralKey(true);
      const trimmedKey = mistralApiKey.trim();

      await graphqlRequest(UPSERT_CONTRAGENT_SETTING_MUTATION, {
        input: {
          key: 'mistral.api.key',
          value: trimmedKey ? trimmedKey : null,
          description: 'Mistral AI API ключ',
          encrypted: true
        }
      });

      await graphqlRequest(UPSERT_CONTRAGENT_SETTING_MUTATION, {
        input: {
          key: 'ai.provider',
          value: 'mistral',
          description: 'Активен AI доставчик за контрагенти'
        }
      });

      alert('Mistral AI ключът е запазен успешно!');
      await loadContragentSettings();
    } catch (err) {
      alert('Грешка при запазване на Mistral ключа: ' + err.message);
    } finally {
      setSavingMistralKey(false);
    }
  };

  const handleProcessGlobalContragents = async () => {
    try {
      setProcessingGlobalContragents(true);
      const response = await graphqlRequest(PROCESS_GLOBAL_CONTRAGENTS_MUTATION);
      const result = response.processGlobalContragents;

      if (result) {
        alert(`Обработени ${result.processed} контрагента. Неуспешни: ${result.failed}.`);
      } else {
        alert('Обработката завърши без резултат.');
      }

      await loadContragentSummary();
    } catch (err) {
      alert('Грешка при обновяване на глобалните контрагенти: ' + err.message);
    } finally {
      setProcessingGlobalContragents(false);
    }
  };

  const handleCreateBackup = async () => {
    try {
      setBackupInProgress(true);
      const response = await graphqlRequest(CREATE_BACKUP_MUTATION);
      const payload = response.createDatabaseBackup;

      if (payload && payload.backup) {
        const createdAt = new Date(payload.backup.createdAt).toLocaleString('bg-BG');
        const durationSeconds = (payload.durationMs / 1000).toFixed(1);
        const remoteNote = payload.remoteObjectKey
          ? `\nS3 ключ: ${payload.remoteObjectKey}`
          : '';
        alert(
          `Резервното копие е създадено успешно!\n\nФайл: ${payload.backup.fileName}\nРазмер: ${payload.backup.sizePretty}\nСъздадено: ${createdAt}\nВреме за изпълнение: ${durationSeconds} сек.${remoteNote}`
        );
      } else {
        alert('Резервното копие е създадено, но липсва информация за файла.');
      }

      await loadDatabaseStatus();
    } catch (err) {
      alert('Грешка при създаване на резервно копие: ' + err.message);
    } finally {
      setBackupInProgress(false);
    }
  };

  const handleOptimizeDatabase = async () => {
    try {
      setOptimizeInProgress(true);
      const response = await graphqlRequest(OPTIMIZE_DATABASE_MUTATION);
      const payload = response.optimizeDatabase;

      if (payload) {
        setLastOptimizationResult(payload);
        const diffBytes = payload.sizeBeforeBytes - payload.sizeAfterBytes;
        const diffLabel = diffBytes > 0 ? `Спестени приблизително ${(diffBytes / (1024 * 1024)).toFixed(2)} MB.` : 'Размерът на базата се запази.';
        const durationSeconds = (payload.durationMs / 1000).toFixed(1);
        alert(
          `Оптимизацията завърши успешно!\n\nПреди: ${payload.sizeBeforePretty}\nСлед: ${payload.sizeAfterPretty}\n${diffLabel}\nВреме за изпълнение: ${durationSeconds} сек.`
        );
      }

      await loadDatabaseStatus();
    } catch (err) {
      alert('Грешка при оптимизация на базата данни: ' + err.message);
    } finally {
      setOptimizeInProgress(false);
    }
  };

  const handleRestoreBackup = async (backup) => {
    if (!backup) return;

    const storageType = backup.storageType;
    const identifier = storageType === 'OBJECT_STORAGE'
      ? (backup.objectKey || backup.fullPath)
      : backup.fullPath;

    if (!identifier) {
      alert('Не може да се определи източникът на архива за възстановяване.');
      return;
    }

    const confirmation = window.confirm(
      `Ще бъде възстановена базата данни от архива "${backup.fileName}" (${storageType === 'OBJECT_STORAGE' ? 'S3' : 'локално'}).\n\nТова действие ще презапише текущите данни. Продължаване?`
    );

    if (!confirmation) return;

    try {
      setRestoreInProgress(backup.fullPath);
      const response = await graphqlRequest(RESTORE_DATABASE_MUTATION, {
        input: {
          storageType,
          identifier,
        },
      });

      const payload = response.restoreDatabase;
      if (payload) {
        const durationSeconds = (payload.durationMs / 1000).toFixed(1);
        alert(
          `Възстановяването завърши успешно!\n\nИзточник: ${payload.source}\nТип: ${payload.storageType}\nВреме за изпълнение: ${durationSeconds} сек.`
        );
      } else {
        alert('Възстановяването е изпълнено, но не беше върната информация.');
      }

      await loadDatabaseStatus();
    } catch (err) {
      alert('Грешка при възстановяване на базата данни: ' + err.message);
    } finally {
      setRestoreInProgress(null);
    }
  };

  const handleObjectStorageFieldChange = (field, value) => {
    setObjectStorageSettings((prev) => ({
      ...prev,
      [field]: value
    }));
  };

  const handleObjectStorageSecretChange = (value) => {
    setObjectStorageSettings((prev) => ({
      ...prev,
      secretKey: value,
      secretProvided: true
    }));
  };

  const handleSaveObjectStorageSettings = async () => {
    try {
      setSavingObjectStorage(true);
      const input = {
        enabled: objectStorageSettings.enabled,
        endpoint: objectStorageSettings.endpoint || null,
        accessKey: objectStorageSettings.accessKey,
        secretKey: objectStorageSettings.secretProvided
          ? objectStorageSettings.secretKey
          : null,
        region: objectStorageSettings.region,
        bucket: objectStorageSettings.bucket,
        prefix: objectStorageSettings.prefix || null,
        forcePathStyle: objectStorageSettings.forcePathStyle
      };

      const response = await graphqlRequest(UPDATE_OBJECT_STORAGE_SETTINGS_MUTATION, {
        input
      });

      const updated = response.updateObjectStorageSettings;
      if (updated) {
        setObjectStorageSettings(() => ({
          enabled: updated.enabled,
          endpoint: updated.endpoint || '',
          accessKey: updated.accessKey || '',
          region: updated.region || 'eu-central',
          bucket: updated.bucket || '',
          prefix: updated.prefix || '',
          forcePathStyle: updated.forcePathStyle ?? true,
          hasSecretKey: updated.hasSecretKey || false,
          secretKey: '',
          secretProvided: false
        }));
        alert('Object Storage настройките са запазени успешно!');
      }

      await loadDatabaseStatus();
    } catch (err) {
      alert('Грешка при запазване на Object Storage настройките: ' + err.message);
    } finally {
      setSavingObjectStorage(false);
      setObjectStorageSettings((prev) => ({ ...prev, secretProvided: false }));
    }
  };

  const formatRelativeTime = (isoString) => {
    if (!isoString) return '—';
    const date = new Date(isoString);
    if (Number.isNaN(date.getTime())) return '—';
    const diff = date.getTime() - Date.now();
    const diffMinutes = Math.round(diff / (1000 * 60));
    if (!Number.isFinite(diffMinutes)) return '—';
    const formatter = new Intl.RelativeTimeFormat('bg', { numeric: 'auto' });

    if (Math.abs(diffMinutes) < 60) {
      return formatter.format(diffMinutes, 'minute');
    }

    const diffHours = Math.round(diffMinutes / 60);
    if (Math.abs(diffHours) < 24) {
      return formatter.format(diffHours, 'hour');
    }

    const diffDays = Math.round(diffHours / 24);
    return formatter.format(diffDays, 'day');
  };

  const lastBackup = databaseStatus?.recentBackups?.[0] || null;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Системни настройки</h1>
          <p className="mt-1 text-sm text-gray-500">
            Общи настройки на системата
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* General settings */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Общи настройки
            </h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-700">
                    Език на системата
                  </label>
                  <p className="text-xs text-gray-500">
                    Основен език за всички потребители
                  </p>
                </div>
                <select className="block px-3 py-2 border border-gray-300 bg-white rounded-md shadow-sm text-sm">
                  <option defaultValue>Български</option>
                  <option>English</option>
                </select>
              </div>
              
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-700">
                    Валута по подразбиране
                  </label>
                  <p className="text-xs text-gray-500">
                    Основна валута за отчитане
                  </p>
                </div>
                <select className="block px-3 py-2 border border-gray-300 bg-white rounded-md shadow-sm text-sm">
                  <option defaultValue>BGN - Български лев</option>
                  <option>EUR - Евро</option>
                  <option>USD - Американски долар</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-700">
                    Автоматично обновяване на БНБ курсове
                  </label>
                  <p className="text-xs text-gray-500">
                    Обновяване всеки ден в 9:00
                  </p>
                </div>
                <label className="inline-flex items-center">
                  <input type="checkbox" className="form-checkbox h-4 w-4 text-blue-600" defaultChecked />
                  <span className="ml-2 text-sm text-gray-700">Активно</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Database settings */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-4 py-5 sm:p-6 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">
                База данни
              </h3>
              {databaseStatus && (
                <div className="text-right">
                  <div className="text-xs text-gray-500">Размер</div>
                  <div className="text-sm font-semibold text-gray-900">
                    {databaseStatus.databaseSizePretty}
                  </div>
                </div>
              )}
            </div>

            {loadingDatabaseStatus ? (
              <div className="text-center py-6">
                <div className="text-sm text-gray-500">Зареждане на информация за базата данни...</div>
              </div>
            ) : (
              <>
                <div className="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
                      <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-green-900">Свързано</p>
                      <p className="text-xs text-green-600">
                        PostgreSQL • {databaseStatus?.databaseName || 'rs-ac-bg'}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Последно резервно копие</span>
                    <span className="font-medium text-gray-900">
                      {lastBackup
                        ? `${formatRelativeTime(lastBackup.createdAt)} (${new Date(lastBackup.createdAt).toLocaleString('bg-BG')})`
                        : 'Няма налично архивно копие'}
                    </span>
                  </div>
                  {lastBackup && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Размер на архива</span>
                      <span className="font-medium text-gray-900">{lastBackup.sizePretty}</span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-gray-600">Директория за архиви</span>
                    <span className="font-medium text-gray-900">
                      {databaseStatus?.backupsDirectory || 'backups'}
                    </span>
                  </div>
                </div>

                <div className="flex flex-col sm:flex-row sm:space-x-2 space-y-2 sm:space-y-0">
                  <button
                    onClick={handleCreateBackup}
                    disabled={backupInProgress}
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {backupInProgress ? 'Създава...' : 'Ново резервно копие'}
                  </button>
                  <button
                    onClick={handleOptimizeDatabase}
                    disabled={optimizeInProgress}
                    className="flex-1 px-3 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {optimizeInProgress ? 'Оптимизира...' : 'Оптимизирай'}
                  </button>
                </div>

                {databaseStatus?.objectStorageEnabled && (
                  <div className="flex items-center text-xs text-blue-700 bg-blue-50 border border-blue-100 rounded-md px-3 py-2">
                    <span className="font-semibold mr-1">Object Storage:</span>
                    <span>Активно – архивите се качват и в S3 съвместимо хранилище.</span>
                  </div>
                )}

                {lastOptimizationResult && (
                  <div className="bg-blue-50 border border-blue-100 rounded-md p-3 text-xs text-blue-900 space-y-1">
                    <div className="font-semibold">Последна оптимизация</div>
                    <div>Преди: {lastOptimizationResult.sizeBeforePretty}</div>
                    <div>След: {lastOptimizationResult.sizeAfterPretty}</div>
                    <div>
                      Време за изпълнение: {(lastOptimizationResult.durationMs / 1000).toFixed(1)} сек.
                      {lastOptimizationResult.reindexRan ? ' • Reindex изпълнен' : ' • Reindex пропуснат'}
                    </div>
                  </div>
                )}

                {databaseStatus?.recentBackups && databaseStatus.recentBackups.length > 0 && (
                  <div className="space-y-2">
                    <div className="text-xs font-semibold text-gray-500 uppercase">
                      Последни архиви
                    </div>
                    <div className="space-y-2">
                      {databaseStatus.recentBackups.slice(0, 5).map((backup) => (
                        <div
                          key={`${backup.fullPath}-${backup.createdAt}`}
                          className="border border-gray-100 rounded-md px-3 py-2 text-xs bg-gray-50 flex flex-col sm:flex-row sm:items-center sm:justify-between"
                        >
                          <div className="space-y-0.5">
                            <div className="text-sm font-semibold text-gray-800 flex items-center space-x-2">
                              <span>{backup.fileName}</span>
                              <span className={`text-[10px] uppercase px-2 py-0.5 rounded-full ${backup.storageType === 'OBJECT_STORAGE' ? 'bg-blue-100 text-blue-700' : 'bg-gray-200 text-gray-700'}`}>
                                {backup.storageType === 'OBJECT_STORAGE' ? 'S3' : 'Локално'}
                              </span>
                            </div>
                            <div className="text-gray-600">
                              {backup.sizePretty} • {new Date(backup.createdAt).toLocaleString('bg-BG')}
                            </div>
                          </div>
                          <div className="mt-2 sm:mt-0 flex items-center space-x-2">
                            <button
                              onClick={() => handleRestoreBackup(backup)}
                              disabled={restoreInProgress === backup.fullPath}
                              className="px-3 py-1 text-xs font-medium border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                              {restoreInProgress === backup.fullPath ? 'Възстановява...' : 'Възстанови'}
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="border-t border-gray-200 pt-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm font-semibold text-gray-900">
                        Object Storage (S3)
                      </div>
                      <p className="text-xs text-gray-500">
                        Настройте Hetzner / S3 съвместимо хранилище за автоматично качване на архивите.
                      </p>
                    </div>
                    <label className="flex items-center space-x-2 text-sm">
                      <span>Активирай</span>
                      <input
                        type="checkbox"
                        className="form-checkbox h-4 w-4 text-blue-600"
                        checked={objectStorageSettings.enabled}
                        onChange={(e) => handleObjectStorageFieldChange('enabled', e.target.checked)}
                        disabled={loadingObjectStorage}
                      />
                    </label>
                  </div>

                  {loadingObjectStorage ? (
                    <div className="text-xs text-gray-500">Зареждане на Object Storage настройките...</div>
                  ) : (
                    <div className="space-y-3">
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Endpoint</label>
                          <input
                            type="text"
                            value={objectStorageSettings.endpoint}
                            onChange={(e) => handleObjectStorageFieldChange('endpoint', e.target.value)}
                            placeholder="https://fsn1.your-object-storage.com"
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            disabled={savingObjectStorage}
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Регион</label>
                          <input
                            type="text"
                            value={objectStorageSettings.region}
                            onChange={(e) => handleObjectStorageFieldChange('region', e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            disabled={savingObjectStorage}
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Bucket</label>
                          <input
                            type="text"
                            value={objectStorageSettings.bucket}
                            onChange={(e) => handleObjectStorageFieldChange('bucket', e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            disabled={savingObjectStorage}
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Prefix (по избор)</label>
                          <input
                            type="text"
                            value={objectStorageSettings.prefix}
                            onChange={(e) => handleObjectStorageFieldChange('prefix', e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            placeholder="rs-ac-bg/backups"
                            disabled={savingObjectStorage}
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Access Key</label>
                          <input
                            type="text"
                            value={objectStorageSettings.accessKey}
                            onChange={(e) => handleObjectStorageFieldChange('accessKey', e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            disabled={savingObjectStorage}
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">Secret Key</label>
                          <input
                            type="password"
                            value={objectStorageSettings.secretKey}
                            placeholder={objectStorageSettings.hasSecretKey ? 'Запазен ключ — оставете празно за без промяна' : ''}
                            onChange={(e) => handleObjectStorageSecretChange(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                            disabled={savingObjectStorage}
                          />
                          {objectStorageSettings.hasSecretKey && !objectStorageSettings.secretProvided && (
                            <p className="text-[10px] text-gray-500 mt-1">Оставете празно, за да запазите текущия ключ.</p>
                          )}
                        </div>
                      </div>

                      <label className="inline-flex items-center space-x-2 text-xs text-gray-600">
                        <input
                          type="checkbox"
                          className="form-checkbox h-4 w-4 text-blue-600"
                          checked={objectStorageSettings.forcePathStyle}
                          onChange={(e) => handleObjectStorageFieldChange('forcePathStyle', e.target.checked)}
                          disabled={savingObjectStorage}
                        />
                        <span>Force path-style URLs (препоръчително за Hetzner)</span>
                      </label>

                      <div className="flex flex-col sm:flex-row sm:items-center sm:space-x-2 space-y-2 sm:space-y-0">
                        <button
                          onClick={handleSaveObjectStorageSettings}
                          disabled={savingObjectStorage}
                          className="w-full sm:w-auto px-4 py-2 bg-purple-600 text-white rounded-md text-sm font-medium hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                          {savingObjectStorage ? 'Запазва...' : 'Запази S3 настройките'}
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
        </div>

        {/* Company-level contragent settings */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-1">
              Настройки за контрагенти по фирма
            </h3>
            <p className="text-xs text-gray-500 mb-4">
              Управлява VIES проверките и AI мапването за избраната компания
            </p>
            <div className="space-y-4">
              {loadingCompanies ? (
                <div className="text-center py-4">
                  <div className="text-sm text-gray-500">Зареждане на фирмите...</div>
                </div>
              ) : (
                <>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Компания
                    </label>
                    <select
                      value={selectedCompanyId || ''}
                      onChange={(e) => handleCompanyChange(e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm"
                    >
                      <option value="">Изберете компания...</option>
                      {companies.map(company => (
                        <option key={company.id} value={company.id}>
                          {company.name}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="space-y-3">
                    <label className="flex items-center justify-between">
                      <div>
                        <span className="text-sm font-medium text-gray-700">
                          VIES валидация
                        </span>
                        <p className="text-xs text-gray-500">
                          Автоматична проверка на ДДС номера при въвеждане
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={enableViesValidation}
                        onChange={(e) => setEnableViesValidation(e.target.checked)}
                        className="form-checkbox h-4 w-4 text-blue-600"
                      />
                    </label>

                    <label className="flex items-center justify-between">
                      <div>
                        <span className="text-sm font-medium text-gray-700">
                          AI мапване на адреси
                        </span>
                        <p className="text-xs text-gray-500">
                          Интелигентно разделяне на адресни компоненти чрез Mistral AI
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={enableAiMapping}
                        onChange={(e) => setEnableAiMapping(e.target.checked)}
                        className="form-checkbox h-4 w-4 text-blue-600"
                      />
                    </label>

                    <label className="flex items-center justify-between">
                      <div>
                        <span className="text-sm font-medium text-gray-700">
                          Валидация при импорт
                        </span>
                        <p className="text-xs text-gray-500">
                          Автоматично проверява и актуализира контрагентите при масов импорт
                        </p>
                      </div>
                      <input
                        type="checkbox"
                        checked={autoValidateOnImport}
                        onChange={(e) => setAutoValidateOnImport(e.target.checked)}
                        className="form-checkbox h-4 w-4 text-blue-600"
                      />
                    </label>
                  </div>

                  <button
                    onClick={handleSaveApiSettings}
                    disabled={saving || !selectedCompanyId}
                    className="w-full px-4 py-2 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 disabled:opacity-50"
                  >
                    {saving ? 'Запазва...' : '💾 Запази настройките'}
                  </button>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Global contragent & AI settings */}
        <div className="bg-white shadow-sm rounded-lg border border-gray-200">
          <div className="px-4 py-5 sm:p-6 space-y-4">
            <div className="flex items-start justify-between">
              <div>
                <h3 className="text-lg font-medium text-gray-900">
                  Глобални контрагенти и Mistral AI
                </h3>
                <p className="text-xs text-gray-500 mt-1">
                  Ключът се използва за адресно мапване и обогатяване на глобалната база с контрагенти
                </p>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Mistral AI API ключ
              </label>
              <input
                type="password"
                value={mistralApiKey}
                onChange={(e) => setMistralApiKey(e.target.value)}
                placeholder="sk-********"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm"
                disabled={loadingContragentSettings}
              />
              <p className="mt-1 text-xs text-gray-500">
                Съхранява се криптирано в системата. Използва се за AI обработка на адреси и данни за контрагенти.
              </p>
            </div>

            <div className="flex flex-col sm:flex-row sm:items-center sm:space-x-3 space-y-2 sm:space-y-0">
              <button
                onClick={handleSaveMistralSettings}
                disabled={savingMistralKey || loadingContragentSettings}
                className="w-full sm:w-auto px-4 py-2 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
              >
                {savingMistralKey ? 'Запазва...' : '💾 Запази Mistral ключа'}
              </button>
              <button
                onClick={handleProcessGlobalContragents}
                disabled={processingGlobalContragents}
                className="w-full sm:w-auto px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
              >
                {processingGlobalContragents ? 'Обновява...' : '🔄 Обнови глобалните контрагенти'}
              </button>
            </div>

            <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
              {loadingSummary ? (
                <div className="text-sm text-gray-500">Зареждане на статистика...</div>
              ) : contragentSummary ? (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
                  <div>
                    <div className="text-xs text-gray-500 uppercase">Общо контрагенти</div>
                    <div className="text-lg font-semibold text-gray-900">{contragentSummary.total}</div>
                  </div>
                  <div>
                    <div className="text-xs text-gray-500 uppercase">Валидни / Невалидни</div>
                    <div className="text-lg font-semibold text-gray-900">
                      {contragentSummary.validCount} / {contragentSummary.invalidCount}
                    </div>
                  </div>
                  <div className="sm:col-span-2">
                    <div className="text-xs text-gray-500 uppercase">Последна синхронизация</div>
                    <div className="text-sm text-gray-900">
                      {contragentSummary.lastSyncedAt
                        ? new Date(contragentSummary.lastSyncedAt).toLocaleString('bg-BG')
                        : 'Няма данни'}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-sm text-gray-500">
                  Все още няма налични данни за глобални контрагенти.
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* System info */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">
            Информация за системата
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">v1.0.0</div>
              <div className="text-sm text-gray-500">Версия</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">99.9%</div>
              <div className="text-sm text-gray-500">Наличност</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-600">3</div>
              <div className="text-sm text-gray-500">Потребители</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600">2</div>
              <div className="text-sm text-gray-500">Фирми</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Settings Navigation Tabs
function SettingsTabs() {
  const location = useLocation();
  
  const tabs = [
    { name: 'Потребители', href: '/settings/users', icon: '👥' },
    { name: 'Фирми', href: '/settings/companies', icon: '🏢' },
    { name: 'Достъп до фирми', href: '/settings/user-companies', icon: '🔗' },
    { name: 'AI Счетоводство', href: '/settings/ai-accounting', icon: '🤖' },
    { name: 'Система', href: '/settings/system', icon: '⚙️' }
  ];

  const isActive = (href) => {
    if (location.pathname === '/settings' && href === '/settings/users') return true;
    return location.pathname.startsWith(href);
  };

  return (
    <div className="border-b border-gray-200 mb-6">
      <nav className="-mb-px flex space-x-8">
        {tabs.map((tab) => (
          <Link
            key={tab.name}
            to={tab.href}
            className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center ${
              isActive(tab.href)
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            <span className="mr-2">{tab.icon}</span>
            {tab.name}
          </Link>
        ))}
      </nav>
    </div>
  );
}

// AI Accounting Settings Wrapper
function AIAccountingSettingsPage() {
  const [selectedCompanyId, setSelectedCompanyId] = useState(null);
  const [companies, setCompanies] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Fetch companies list
    const fetchCompanies = async () => {
      try {
        const query = `
          query {
            companies {
              id
              name
              eik
            }
          }
        `;
        const data = await graphqlRequest(query);
        setCompanies(data.companies || []);

        // Try to get last selected company from localStorage
        const lastCompanyId = localStorage.getItem('selectedCompanyId');
        if (lastCompanyId && data.companies.find(c => c.id === parseInt(lastCompanyId))) {
          setSelectedCompanyId(parseInt(lastCompanyId));
        } else if (data.companies.length > 0) {
          setSelectedCompanyId(data.companies[0].id);
        }
      } catch (err) {
        console.error('Error fetching companies:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchCompanies();
  }, []);

  const handleCompanyChange = (companyId) => {
    setSelectedCompanyId(companyId);
    localStorage.setItem('selectedCompanyId', companyId.toString());
  };

  if (loading) {
    return <div className="flex justify-center items-center h-64">Зареждане...</div>;
  }

  return (
    <div className="space-y-4">
      {/* Company Selector */}
      {companies.length > 0 && (
        <div className="bg-white p-4 rounded-lg shadow">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Изберете фирма
          </label>
          <select
            value={selectedCompanyId || ''}
            onChange={(e) => handleCompanyChange(parseInt(e.target.value))}
            className="block w-full md:w-96 px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
          >
            {companies.map((company) => (
              <option key={company.id} value={company.id}>
                {company.name} ({company.eik})
              </option>
            ))}
          </select>
        </div>
      )}

      {/* AI Accounting Settings Component */}
      {selectedCompanyId ? (
        <AIAccountingSettings companyId={selectedCompanyId} />
      ) : (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <p className="text-yellow-800">Моля, изберете фирма за да конфигурирате настройките</p>
        </div>
      )}
    </div>
  );
}

// Main Settings Component
export default function Settings() {
  return (
    <div className="space-y-6">
      <SettingsTabs />

      <Routes>
        <Route path="/" element={<UsersPage />} />
        <Route path="/users" element={<UsersPage />} />
        <Route path="/companies" element={<CompaniesPage />} />
        <Route path="/user-companies" element={<AdminUserCompanies />} />
        <Route path="/ai-accounting" element={<AIAccountingSettingsPage />} />
        <Route path="/system" element={<SystemPage />} />
      </Routes>
    </div>
  );
}
