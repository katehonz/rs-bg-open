import { useState, useEffect } from 'react';
import { graphqlRequest } from '../utils/graphqlClient';

// Simple User-Company Management Component for Settings
export default function AdminUserCompanies() {
  const [userCompanies, setUserCompanies] = useState([]);
  const [users, setUsers] = useState([]);
  const [companies, setCompanies] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showAssignModal, setShowAssignModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedUserCompany, setSelectedUserCompany] = useState(null);

  const GET_USER_COMPANIES_QUERY = `
    query GetUserCompanies {
      userCompanies {
        id
        userId
        companyId
        role
        isActive
        createdAt
        user {
          id
          username
          firstName
          lastName
          email
        }
        company {
          id
          name
          eik
        }
      }
    }
  `;

  const GET_USERS_QUERY = `
    query GetUsers {
      users {
        id
        username
        firstName
        lastName
        email
        isActive
      }
    }
  `;

  const GET_COMPANIES_QUERY = `
    query GetCompanies {
      companies {
        id
        name
        eik
        isActive
      }
    }
  `;

  const ASSIGN_USER_MUTATION = `
    mutation AssignUserToCompany($input: CreateUserCompanyInput!) {
      assignUserToCompany(input: $input) {
        id
        userId
        companyId
        role
        isActive
      }
    }
  `;

  const UPDATE_USER_COMPANY_MUTATION = `
    mutation UpdateUserCompany($id: Int!, $input: UpdateUserCompanyInput!) {
      updateUserCompany(id: $id, input: $input) {
        id
        role
        isActive
      }
    }
  `;

  const REMOVE_USER_COMPANY_MUTATION = `
    mutation RemoveUserFromCompany($id: Int!) {
      removeUserFromCompany(id: $id)
    }
  `;

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [userCompaniesResult, usersResult, companiesResult] = await Promise.all([
        graphqlRequest(GET_USER_COMPANIES_QUERY),
        graphqlRequest(GET_USERS_QUERY),
        graphqlRequest(GET_COMPANIES_QUERY)
      ]);

      setUserCompanies(userCompaniesResult.userCompanies || []);
      setUsers(usersResult.users || []);
      setCompanies(companiesResult.companies || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleAssignUser = async (formData) => {
    try {
      await graphqlRequest(ASSIGN_USER_MUTATION, { input: formData });
      setShowAssignModal(false);
      await loadData();
      alert('Потребителят е закачен към фирмата успешно!');
    } catch (err) {
      alert('Грешка: ' + err.message);
    }
  };

  const handleUpdateUserCompany = async (formData) => {
    try {
      const { userId: _userId, companyId: _companyId, ...updateData } = formData;
      await graphqlRequest(UPDATE_USER_COMPANY_MUTATION, {
        id: selectedUserCompany.id,
        input: updateData
      });
      setShowEditModal(false);
      setSelectedUserCompany(null);
      await loadData();
      alert('Връзката е обновена успешно!');
    } catch (err) {
      alert('Грешка: ' + err.message);
    }
  };

  const handleRemoveUserCompany = async (userCompany) => {
    if (confirm(`Сигурни ли сте, че искате да премахнете достъпа на ${userCompany.user.firstName} ${userCompany.user.lastName} към ${userCompany.company.name}?`)) {
      try {
        await graphqlRequest(REMOVE_USER_COMPANY_MUTATION, { id: userCompany.id });
        await loadData();
        alert('Потребителят е премахнат от фирмата успешно!');
      } catch (err) {
        alert('Грешка: ' + err.message);
      }
    }
  };

  const getRoleLabel = (role) => {
    const roleLabels = {
      admin: 'Администратор',
      user: 'Потребител',
      viewer: 'Наблюдател'
    };
    return roleLabels[role] || role;
  };

  if (loading) return <div className="text-center py-4">Зареждане...</div>;
  if (error) return <div className="text-center py-4 text-red-600">Грешка: {error}</div>;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Достъп до фирми</h3>
          <p className="mt-1 text-sm text-gray-500">
            Управлявайте кой потребител има достъп до коя фирма
          </p>
        </div>
        <button
          onClick={() => setShowAssignModal(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700"
        >
          🔗 Закачи потребител
        </button>
      </div>

      {/* User-Company Table */}
      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Потребител
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Фирма
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Роля
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Статус
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Действия
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {userCompanies.map((userCompany) => (
              <tr key={userCompany.id}>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="flex items-center">
                    <div>
                      <div className="text-sm font-medium text-gray-900">
                        {userCompany.user.firstName} {userCompany.user.lastName}
                      </div>
                      <div className="text-sm text-gray-500">
                        {userCompany.user.username} • {userCompany.user.email}
                      </div>
                    </div>
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm font-medium text-gray-900">
                    {userCompany.company.name}
                  </div>
                  <div className="text-sm text-gray-500">
                    ЕИК: {userCompany.company.eik}
                  </div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                    userCompany.role === 'admin' ? 'bg-red-100 text-red-800' :
                    userCompany.role === 'user' ? 'bg-blue-100 text-blue-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {getRoleLabel(userCompany.role)}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                    userCompany.isActive ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                  }`}>
                    {userCompany.isActive ? 'Активна' : 'Неактивна'}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <button
                    onClick={() => {
                      setSelectedUserCompany(userCompany);
                      setShowEditModal(true);
                    }}
                    className="text-indigo-600 hover:text-indigo-900 mr-4"
                  >
                    Редактиране
                  </button>
                  <button
                    onClick={() => handleRemoveUserCompany(userCompany)}
                    className="text-red-600 hover:text-red-900"
                  >
                    Премахване
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {userCompanies.length === 0 && (
          <div className="text-center py-8 text-gray-500">
            Няма закачени потребители към фирми
          </div>
        )}
      </div>

      {/* Assign User Modal */}
      {showAssignModal && (
        <AssignUserModal
          users={users}
          companies={companies}
          onClose={() => setShowAssignModal(false)}
          onSubmit={handleAssignUser}
        />
      )}

      {/* Edit User-Company Modal */}
      {showEditModal && selectedUserCompany && (
        <EditUserCompanyModal
          userCompany={selectedUserCompany}
          onClose={() => {
            setShowEditModal(false);
            setSelectedUserCompany(null);
          }}
          onSubmit={handleUpdateUserCompany}
        />
      )}
    </div>
  );
}

// Assign User Modal Component
function AssignUserModal({ users, companies, onClose, onSubmit }) {
  const [formData, setFormData] = useState({
    userId: '',
    companyId: '',
    role: 'user',
    isActive: true
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    if (!formData.userId || !formData.companyId) {
      alert('Моля изберете потребител и фирма');
      return;
    }
    onSubmit({
      ...formData,
      userId: parseInt(formData.userId),
      companyId: parseInt(formData.companyId)
    });
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-1/2 shadow-lg rounded-md bg-white">
        <h3 className="text-lg font-medium text-gray-900 mb-4">
          Закачване на потребител към фирма
        </h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700">Потребител</label>
            <select
              value={formData.userId}
              onChange={(e) => setFormData({ ...formData, userId: e.target.value })}
              className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
              required
            >
              <option value="">Изберете потребител</option>
              {users.filter(u => u.isActive).map(user => (
                <option key={user.id} value={user.id}>
                  {user.firstName} {user.lastName} ({user.username})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700">Фирма</label>
            <select
              value={formData.companyId}
              onChange={(e) => setFormData({ ...formData, companyId: e.target.value })}
              className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
              required
            >
              <option value="">Изберете фирма</option>
              {companies.filter(c => c.isActive).map(company => (
                <option key={company.id} value={company.id}>
                  {company.name} ({company.eik})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700">Роля</label>
            <select
              value={formData.role}
              onChange={(e) => setFormData({ ...formData, role: e.target.value })}
              className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
            >
              <option value="admin">Администратор</option>
              <option value="user">Потребител</option>
              <option value="viewer">Наблюдател</option>
            </select>
          </div>

          <div className="flex items-center">
            <input
              type="checkbox"
              checked={formData.isActive}
              onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
              className="h-4 w-4 text-indigo-600"
            />
            <label className="ml-2 block text-sm text-gray-900">
              Активна връзка
            </label>
          </div>

          <div className="flex justify-end space-x-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
            >
              Отказ
            </button>
            <button
              type="submit"
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Закачване
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Edit User-Company Modal Component
function EditUserCompanyModal({ userCompany, onClose, onSubmit }) {
  const [formData, setFormData] = useState({
    role: userCompany.role,
    isActive: userCompany.isActive
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(formData);
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 md:w-1/2 shadow-lg rounded-md bg-white">
        <h3 className="text-lg font-medium text-gray-900 mb-4">
          Редактиране на връзка
        </h3>
        <div className="mb-4 p-4 bg-gray-50 rounded-md">
          <p><strong>Потребител:</strong> {userCompany.user.firstName} {userCompany.user.lastName}</p>
          <p><strong>Фирма:</strong> {userCompany.company.name}</p>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700">Роля</label>
            <select
              value={formData.role}
              onChange={(e) => setFormData({ ...formData, role: e.target.value })}
              className="mt-1 block w-full border border-gray-300 rounded-md px-3 py-2"
            >
              <option value="admin">Администратор</option>
              <option value="user">Потребител</option>
              <option value="viewer">Наблюдател</option>
            </select>
          </div>

          <div className="flex items-center">
            <input
              type="checkbox"
              checked={formData.isActive}
              onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
              className="h-4 w-4 text-indigo-600"
            />
            <label className="ml-2 block text-sm text-gray-900">
              Активна връзка
            </label>
          </div>

          <div className="flex justify-end space-x-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
            >
              Отказ
            </button>
            <button
              type="submit"
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Запазване
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
