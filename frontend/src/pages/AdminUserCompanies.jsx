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
  CardContent,
  IconButton
} from '@mui/material';
import {
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Person as PersonIcon,
  Business as BusinessIcon,
  Link as LinkIcon,
  LinkOff as LinkOffIcon
} from '@mui/icons-material';

const GET_USERS = gql`
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

const GET_COMPANIES = gql`
  query GetCompanies {
    companies {
      id
      name
      eik
      isActive
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

const ASSIGN_USER_TO_COMPANY = gql`
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

const UPDATE_USER_COMPANY = gql`
  mutation UpdateUserCompany($id: Int!, $input: UpdateUserCompanyInput!) {
    updateUserCompany(id: $id, input: $input) {
      id
      role
      isActive
    }
  }
`;

const REMOVE_USER_FROM_COMPANY = gql`
  mutation RemoveUserFromCompany($id: Int!) {
    removeUserFromCompany(id: $id)
  }
`;

const UserCompanyForm = ({ open, handleClose, userCompany, users, companies, onSubmit, isEdit = false }) => {
  const [formData, setFormData] = useState({
    userId: '',
    companyId: '',
    role: 'user',
    isActive: true
  });

  useEffect(() => {
    if (userCompany && isEdit) {
      setFormData({
        userId: userCompany.userId,
        companyId: userCompany.companyId,
        role: userCompany.role,
        isActive: userCompany.isActive
      });
    } else if (!isEdit) {
      setFormData({
        userId: '',
        companyId: '',
        role: 'user',
        isActive: true
      });
    }
  }, [userCompany, isEdit, open]);

  const handleSubmit = () => {
    onSubmit(formData);
  };

  const roleOptions = [
    { value: 'admin', label: 'Администратор' },
    { value: 'user', label: 'Потребител' },
    { value: 'viewer', label: 'Наблюдател' }
  ];

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>{isEdit ? 'Редактиране на връзка' : 'Закачване на потребител към фирма'}</DialogTitle>
      <DialogContent>
        <Grid container spacing={2} sx={{ mt: 1 }}>
          <Grid item xs={12}>
            <TextField
              label="Потребител"
              select
              fullWidth
              value={formData.userId}
              onChange={(e) => setFormData({ ...formData, userId: parseInt(e.target.value) })}
              disabled={isEdit}
              variant="outlined"
            >
              {users.map((user) => (
                <MenuItem key={user.id} value={user.id}>
                  {user.firstName} {user.lastName} ({user.username})
                </MenuItem>
              ))}
            </TextField>
          </Grid>
          <Grid item xs={12}>
            <TextField
              label="Фирма"
              select
              fullWidth
              value={formData.companyId}
              onChange={(e) => setFormData({ ...formData, companyId: parseInt(e.target.value) })}
              disabled={isEdit}
              variant="outlined"
            >
              {companies.map((company) => (
                <MenuItem key={company.id} value={company.id}>
                  {company.name} ({company.eik})
                </MenuItem>
              ))}
            </TextField>
          </Grid>
          <Grid item xs={12}>
            <TextField
              label="Роля"
              select
              fullWidth
              value={formData.role}
              onChange={(e) => setFormData({ ...formData, role: e.target.value })}
              variant="outlined"
            >
              {roleOptions.map((option) => (
                <MenuItem key={option.value} value={option.value}>
                  {option.label}
                </MenuItem>
              ))}
            </TextField>
          </Grid>
          <Grid item xs={12}>
            <FormControlLabel
              control={
                <Switch
                  checked={formData.isActive}
                  onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
                />
              }
              label="Активна връзка"
            />
          </Grid>
        </Grid>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Отказ</Button>
        <Button onClick={handleSubmit} variant="contained">
          {isEdit ? 'Запазване' : 'Закачване'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

const AdminUserCompanies = () => {
  const [openDialog, setOpenDialog] = useState(false);
  const [editUserCompany, setEditUserCompany] = useState(null);
  const [deleteUserCompany, setDeleteUserCompany] = useState(null);
  const [alert, setAlert] = useState(null);

  const { data: usersData, loading: usersLoading } = useQuery(GET_USERS);
  const { data: companiesData, loading: companiesLoading } = useQuery(GET_COMPANIES);
  const {
    data: userCompaniesData,
    loading: userCompaniesLoading,
    error: userCompaniesError,
    refetch: refetchUserCompanies
  } = useQuery(GET_USER_COMPANIES);

  const [assignUserToCompany] = useMutation(ASSIGN_USER_TO_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Потребителят е закачен към фирмата успешно!' });
      setOpenDialog(false);
      refetchUserCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [updateUserCompany] = useMutation(UPDATE_USER_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Връзката е обновена успешно!' });
      setOpenDialog(false);
      setEditUserCompany(null);
      refetchUserCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [removeUserFromCompany] = useMutation(REMOVE_USER_FROM_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Потребителят е премахнат от фирмата успешно!' });
      setDeleteUserCompany(null);
      refetchUserCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const handleAssignUser = (formData) => {
    assignUserToCompany({
      variables: {
        input: formData
      }
    });
  };

  const handleUpdateUserCompany = (formData) => {
    const { userId: _userId, companyId: _companyId, ...updateData } = formData; // Remove immutable fields
    updateUserCompany({
      variables: {
        id: editUserCompany.id,
        input: updateData
      }
    });
  };

  const handleRemoveUser = () => {
    removeUserFromCompany({
      variables: {
        id: deleteUserCompany.id
      }
    });
  };

  const getRoleLabel = (role) => {
    const roleLabels = {
      admin: 'Администратор',
      user: 'Потребител',
      viewer: 'Наблюдател'
    };
    return roleLabels[role] || role;
  };

  const getRoleColor = (role) => {
    const roleColors = {
      admin: 'error',
      user: 'primary',
      viewer: 'default'
    };
    return roleColors[role] || 'default';
  };

  if (usersLoading || companiesLoading || userCompaniesLoading) {
    return <div>Зареждане...</div>;
  }

  if (userCompaniesError) {
    return <Alert severity="error">Грешка при зареждане на данните: {userCompaniesError.message}</Alert>;
  }

  return (
    <Container maxWidth="xl">
      <Box sx={{ mt: 3, mb: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom>
          Управление на достъп до фирми
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
          startIcon={<LinkIcon />}
          onClick={() => {
            setEditUserCompany(null);
            setOpenDialog(true);
          }}
          sx={{ mb: 2 }}
        >
          Закачи потребител към фирма
        </Button>

        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Потребител</TableCell>
                <TableCell>Фирма</TableCell>
                <TableCell>Роля</TableCell>
                <TableCell>Статус</TableCell>
                <TableCell>Създадено</TableCell>
                <TableCell>Действия</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {userCompaniesData?.userCompanies?.map((userCompany) => (
                <TableRow key={userCompany.id}>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <PersonIcon color="action" />
                      <Box>
                        <Typography variant="body2" fontWeight="medium">
                          {userCompany.user.firstName} {userCompany.user.lastName}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {userCompany.user.username} • {userCompany.user.email}
                        </Typography>
                      </Box>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <BusinessIcon color="action" />
                      <Box>
                        <Typography variant="body2" fontWeight="medium">
                          {userCompany.company.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          ЕИК: {userCompany.company.eik}
                        </Typography>
                      </Box>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={getRoleLabel(userCompany.role)}
                      color={getRoleColor(userCompany.role)}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={userCompany.isActive ? 'Активна' : 'Неактивна'}
                      color={userCompany.isActive ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    {new Date(userCompany.createdAt).toLocaleDateString('bg-BG')}
                  </TableCell>
                  <TableCell>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setEditUserCompany(userCompany);
                        setOpenDialog(true);
                      }}
                    >
                      <EditIcon />
                    </IconButton>
                    <IconButton
                      size="small"
                      color="error"
                      onClick={() => setDeleteUserCompany(userCompany)}
                    >
                      <LinkOffIcon />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Box>

      {/* Assign/Edit User-Company Dialog */}
      <UserCompanyForm
        open={openDialog}
        handleClose={() => {
          setOpenDialog(false);
          setEditUserCompany(null);
        }}
        userCompany={editUserCompany}
        users={usersData?.users || []}
        companies={companiesData?.companies || []}
        onSubmit={editUserCompany ? handleUpdateUserCompany : handleAssignUser}
        isEdit={!!editUserCompany}
      />

      {/* Remove Confirmation Dialog */}
      <Dialog open={!!deleteUserCompany} onClose={() => setDeleteUserCompany(null)}>
        <DialogTitle>Потвърждение за премахване</DialogTitle>
        <DialogContent>
          <Typography>
            Сигурни ли сте, че искате да премахнете достъпа на потребител{' '}
            <strong>{deleteUserCompany?.user?.firstName} {deleteUserCompany?.user?.lastName}</strong>{' '}
            към фирма{' '}
            <strong>{deleteUserCompany?.company?.name}</strong>?
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteUserCompany(null)}>Отказ</Button>
          <Button onClick={handleRemoveUser} color="error" variant="contained">
            Премахване
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
};

export default AdminUserCompanies;
