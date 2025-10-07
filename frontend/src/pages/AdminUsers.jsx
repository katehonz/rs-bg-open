import React, { useState, useEffect } from 'react';
import { gql, useQuery, useMutation } from '@apollo/client';
import {
  Container,
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  TextField,
  MenuItem,
  Switch,
  FormControlLabel,
  Box,
  Alert,
  Chip,
  Grid,
  Card,
  CardContent
} from '@mui/material';
import { Add as AddIcon, Edit as EditIcon, Delete as DeleteIcon, Business as BusinessIcon } from '@mui/icons-material';

const GET_USERS = gql`
  query GetUsers {
    users {
      id
      username
      email
      firstName
      lastName
      groupId
      isActive
      createdAt
    }
  }
`;

const GET_USER_GROUPS = gql`
  query GetUserGroups {
    userGroups {
      id
      name
      description
      canManageUsers
      canCreateCompanies
    }
  }
`;

const GET_USER_COMPANIES = gql`
  query GetUserCompanies {
    userCompanies {
      id
      userId
      companyId
      role
      isActive
      user {
        id
        username
        firstName
        lastName
      }
      company {
        id
        name
        eik
      }
    }
  }
`;

const CREATE_USER = gql`
  mutation CreateUser($input: CreateUserInput!) {
    createUser(input: $input) {
      id
      username
      email
      firstName
      lastName
      isActive
    }
  }
`;

const UPDATE_USER = gql`
  mutation UpdateUser($id: Int!, $input: UpdateUserInput!) {
    updateUser(id: $id, input: $input) {
      id
      username
      email
      firstName
      lastName
      isActive
    }
  }
`;

const DELETE_USER = gql`
  mutation DeleteUser($id: Int!) {
    deleteUser(id: $id)
  }
`;

const ASSIGN_USER_TO_COMPANY = gql`
  mutation AssignUserToCompany($input: CreateUserCompanyInput!) {
    assignUserToCompany(input: $input) {
      id
      role
      isActive
    }
  }
`;

const UserForm = ({ open, handleClose, user, userGroups, onSubmit, isEdit = false }) => {
  const [formData, setFormData] = useState({
    username: '',
    email: '',
    password: '',
    firstName: '',
    lastName: '',
    groupId: 1,
    isActive: true
  });

  useEffect(() => {
    if (user && isEdit) {
      setFormData({
        username: user.username || '',
        email: user.email || '',
        password: '',
        firstName: user.firstName || '',
        lastName: user.lastName || '',
        groupId: user.groupId || 1,
        isActive: user.isActive !== undefined ? user.isActive : true
      });
    } else if (!isEdit) {
      setFormData({
        username: '',
        email: '',
        password: '',
        firstName: '',
        lastName: '',
        groupId: 1,
        isActive: true
      });
    }
  }, [user, isEdit, open]);

  const handleSubmit = () => {
    onSubmit(formData);
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <DialogTitle>{isEdit ? 'Редактиране на потребител' : 'Нов потребител'}</DialogTitle>
      <DialogContent>
        <Grid container spacing={2}>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Потребителско име"
              fullWidth
              variant="outlined"
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Email"
              type="email"
              fullWidth
              variant="outlined"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            />
          </Grid>
          {!isEdit && (
            <Grid item xs={12}>
              <TextField
                margin="dense"
                label="Парола"
                type="password"
                fullWidth
                variant="outlined"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              />
            </Grid>
          )}
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Име"
              fullWidth
              variant="outlined"
              value={formData.firstName}
              onChange={(e) => setFormData({ ...formData, firstName: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Фамилия"
              fullWidth
              variant="outlined"
              value={formData.lastName}
              onChange={(e) => setFormData({ ...formData, lastName: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Потребителска група"
              select
              fullWidth
              variant="outlined"
              value={formData.groupId}
              onChange={(e) => setFormData({ ...formData, groupId: parseInt(e.target.value) })}
            >
              {userGroups.map((group) => (
                <MenuItem key={group.id} value={group.id}>
                  {group.name}
                </MenuItem>
              ))}
            </TextField>
          </Grid>
          <Grid item xs={6}>
            <FormControlLabel
              control={
                <Switch
                  checked={formData.isActive}
                  onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
                />
              }
              label="Активен"
              sx={{ mt: 2 }}
            />
          </Grid>
        </Grid>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Отказ</Button>
        <Button onClick={handleSubmit} variant="contained">
          {isEdit ? 'Запазване' : 'Създаване'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

const AdminUsers = () => {
  const [openDialog, setOpenDialog] = useState(false);
  const [editUser, setEditUser] = useState(null);
  const [deleteUser, setDeleteUser] = useState(null);
  const [alert, setAlert] = useState(null);

  const { data: usersData, loading: usersLoading, error: usersError, refetch: refetchUsers } = useQuery(GET_USERS);
  const { data: groupsData, loading: groupsLoading } = useQuery(GET_USER_GROUPS);
  const { data: userCompaniesData, loading: userCompaniesLoading } = useQuery(GET_USER_COMPANIES);

  const [createUser] = useMutation(CREATE_USER, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Потребителят е създаден успешно!' });
      setOpenDialog(false);
      refetchUsers();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [updateUser] = useMutation(UPDATE_USER, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Потребителят е обновен успешно!' });
      setOpenDialog(false);
      setEditUser(null);
      refetchUsers();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [deleteUserMutation] = useMutation(DELETE_USER, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Потребителят е изтрит успешно!' });
      setDeleteUser(null);
      refetchUsers();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const handleCreateUser = (formData) => {
    createUser({
      variables: {
        input: formData
      }
    });
  };

  const handleUpdateUser = (formData) => {
    const { password: _password, ...updateData } = formData; // Remove password from update
    updateUser({
      variables: {
        id: editUser.id,
        input: updateData
      }
    });
  };

  const handleDeleteUser = () => {
    deleteUserMutation({
      variables: {
        id: deleteUser.id
      }
    });
  };

  const getUserCompanies = (userId) => {
    if (!userCompaniesData) return [];
    return userCompaniesData.userCompanies.filter(uc => uc.userId === userId && uc.isActive);
  };

  if (usersLoading || groupsLoading || userCompaniesLoading) {
    return <div>Зареждане...</div>;
  }

  if (usersError) {
    return <Alert severity="error">Грешка при зареждане на данните: {usersError.message}</Alert>;
  }

  return (
    <Container maxWidth="xl">
      <Box sx={{ mt: 3, mb: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom>
          Управление на потребители
        </Typography>

        {alert && (
          <Alert
            severity={alert.type}
            onClose={() => setAlert(null)}
            sx={{ mb: 2 }}
          >
            {alert.message}
          </Alert>
        )}

        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => {
            setEditUser(null);
            setOpenDialog(true);
          }}
          sx={{ mb: 2 }}
        >
          Добави потребител
        </Button>

        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>ID</TableCell>
                <TableCell>Потребителско име</TableCell>
                <TableCell>Име</TableCell>
                <TableCell>Email</TableCell>
                <TableCell>Група</TableCell>
                <TableCell>Компании</TableCell>
                <TableCell>Статус</TableCell>
                <TableCell>Действия</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {usersData?.users?.map((user) => {
                const userGroup = groupsData?.userGroups?.find(g => g.id === user.groupId);
                const companies = getUserCompanies(user.id);

                return (
                  <TableRow key={user.id}>
                    <TableCell>{user.id}</TableCell>
                    <TableCell>{user.username}</TableCell>
                    <TableCell>{`${user.firstName} ${user.lastName}`}</TableCell>
                    <TableCell>{user.email}</TableCell>
                    <TableCell>{userGroup?.name || 'N/A'}</TableCell>
                    <TableCell>
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                        {companies.map((uc) => (
                          <Chip
                            key={uc.id}
                            label={`${uc.company.name} (${uc.role})`}
                            size="small"
                            icon={<BusinessIcon />}
                            variant="outlined"
                          />
                        ))}
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={user.isActive ? 'Активен' : 'Неактивен'}
                        color={user.isActive ? 'success' : 'default'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell>
                      <Button
                        size="small"
                        startIcon={<EditIcon />}
                        onClick={() => {
                          setEditUser(user);
                          setOpenDialog(true);
                        }}
                      >
                        Редактиране
                      </Button>
                      <Button
                        size="small"
                        color="error"
                        startIcon={<DeleteIcon />}
                        onClick={() => setDeleteUser(user)}
                        sx={{ ml: 1 }}
                      >
                        Изтриване
                      </Button>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </Box>

      {/* Create/Edit User Dialog */}
      <UserForm
        open={openDialog}
        handleClose={() => {
          setOpenDialog(false);
          setEditUser(null);
        }}
        user={editUser}
        userGroups={groupsData?.userGroups || []}
        onSubmit={editUser ? handleUpdateUser : handleCreateUser}
        isEdit={!!editUser}
      />

      {/* Delete Confirmation Dialog */}
      <Dialog open={!!deleteUser} onClose={() => setDeleteUser(null)}>
        <DialogTitle>Потвърждение за изтриване</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Сигурни ли сте, че искате да изтриете потребителя "{deleteUser?.username}"?
            Това действие не може да бъде отменено.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteUser(null)}>Отказ</Button>
          <Button onClick={handleDeleteUser} color="error" variant="contained">
            Изтриване
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
};

export default AdminUsers;
