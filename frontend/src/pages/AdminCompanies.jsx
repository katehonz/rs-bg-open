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
  Switch,
  FormControlLabel,
  Box,
  Alert,
  Chip,
  Grid,
  Card,
  CardContent
} from '@mui/material';
import { Add as AddIcon, Edit as EditIcon, Delete as DeleteIcon, Person as PersonIcon } from '@mui/icons-material';

const GET_COMPANIES = gql`
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
      isActive
      createdAt
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

const CREATE_COMPANY = gql`
  mutation CreateCompany($input: CreateCompanyInput!) {
    createCompany(input: $input) {
      id
      name
      eik
      vatNumber
      isActive
    }
  }
`;

const UPDATE_COMPANY = gql`
  mutation UpdateCompany($id: Int!, $input: UpdateCompanyInput!) {
    updateCompany(id: $id, input: $input) {
      id
      name
      eik
      vatNumber
      isActive
    }
  }
`;

const DELETE_COMPANY = gql`
  mutation DeleteCompany($id: Int!) {
    deleteCompany(id: $id)
  }
`;

const CompanyForm = ({ open, handleClose, company, onSubmit, isEdit = false }) => {
  const [formData, setFormData] = useState({
    name: '',
    eik: '',
    vatNumber: '',
    address: '',
    city: '',
    country: 'Bulgaria',
    phone: '',
    email: '',
    contactPerson: '',
    managerName: '',
    authorizedPerson: '',
    managerEgn: '',
    authorizedPersonEgn: '',
    isActive: true
  });

  useEffect(() => {
    if (company && isEdit) {
      setFormData({
        name: company.name || '',
        eik: company.eik || '',
        vatNumber: company.vatNumber || '',
        address: company.address || '',
        city: company.city || '',
        country: company.country || 'Bulgaria',
        phone: company.phone || '',
        email: company.email || '',
        contactPerson: company.contactPerson || '',
        managerName: company.managerName || '',
        authorizedPerson: company.authorizedPerson || '',
        managerEgn: company.managerEgn || '',
        authorizedPersonEgn: company.authorizedPersonEgn || '',
        isActive: company.isActive !== undefined ? company.isActive : true
      });
    } else if (!isEdit) {
      setFormData({
        name: '',
        eik: '',
        vatNumber: '',
        address: '',
        city: '',
        country: 'Bulgaria',
        phone: '',
        email: '',
        contactPerson: '',
        managerName: '',
        authorizedPerson: '',
        managerEgn: '',
        authorizedPersonEgn: '',
        isActive: true
      });
    }
  }, [company, isEdit, open]);

  const handleSubmit = () => {
    onSubmit(formData);
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <DialogTitle>{isEdit ? 'Редактиране на фирма' : 'Нова фирма'}</DialogTitle>
      <DialogContent>
        <Grid container spacing={2}>
          <Grid item xs={8}>
            <TextField
              margin="dense"
              label="Наименование на фирмата"
              fullWidth
              variant="outlined"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
          </Grid>
          <Grid item xs={4}>
            <TextField
              margin="dense"
              label="ЕИК"
              fullWidth
              variant="outlined"
              value={formData.eik}
              onChange={(e) => setFormData({ ...formData, eik: e.target.value })}
              required
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="ДДС номер"
              fullWidth
              variant="outlined"
              value={formData.vatNumber}
              onChange={(e) => setFormData({ ...formData, vatNumber: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Контактно лице"
              fullWidth
              variant="outlined"
              value={formData.contactPerson}
              onChange={(e) => setFormData({ ...formData, contactPerson: e.target.value })}
            />
          </Grid>
          <Grid item xs={12}>
            <TextField
              margin="dense"
              label="Адрес"
              fullWidth
              variant="outlined"
              value={formData.address}
              onChange={(e) => setFormData({ ...formData, address: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Град"
              fullWidth
              variant="outlined"
              value={formData.city}
              onChange={(e) => setFormData({ ...formData, city: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Държава"
              fullWidth
              variant="outlined"
              value={formData.country}
              onChange={(e) => setFormData({ ...formData, country: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Телефон"
              fullWidth
              variant="outlined"
              value={formData.phone}
              onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
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
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Управител"
              fullWidth
              variant="outlined"
              value={formData.managerName}
              onChange={(e) => setFormData({ ...formData, managerName: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="ЕГН на управителя"
              fullWidth
              variant="outlined"
              value={formData.managerEgn}
              onChange={(e) => setFormData({ ...formData, managerEgn: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="Упълномощено лице"
              fullWidth
              variant="outlined"
              value={formData.authorizedPerson}
              onChange={(e) => setFormData({ ...formData, authorizedPerson: e.target.value })}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              margin="dense"
              label="ЕГН на упълномощеното лице"
              fullWidth
              variant="outlined"
              value={formData.authorizedPersonEgn}
              onChange={(e) => setFormData({ ...formData, authorizedPersonEgn: e.target.value })}
            />
          </Grid>
          <Grid item xs={12}>
            <FormControlLabel
              control={
                <Switch
                  checked={formData.isActive}
                  onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
                />
              }
              label="Активна"
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

const AdminCompanies = () => {
  const [openDialog, setOpenDialog] = useState(false);
  const [editCompany, setEditCompany] = useState(null);
  const [deleteCompany, setDeleteCompany] = useState(null);
  const [alert, setAlert] = useState(null);

  const { data: companiesData, loading: companiesLoading, error: companiesError, refetch: refetchCompanies } = useQuery(GET_COMPANIES);
  const { data: userCompaniesData, loading: userCompaniesLoading } = useQuery(GET_USER_COMPANIES);

  const [createCompany] = useMutation(CREATE_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Фирмата е създадена успешно!' });
      setOpenDialog(false);
      refetchCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [updateCompany] = useMutation(UPDATE_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Фирмата е обновена успешно!' });
      setOpenDialog(false);
      setEditCompany(null);
      refetchCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const [deleteCompanyMutation] = useMutation(DELETE_COMPANY, {
    onCompleted: () => {
      setAlert({ type: 'success', message: 'Фирмата е изтрита успешно!' });
      setDeleteCompany(null);
      refetchCompanies();
    },
    onError: (error) => {
      setAlert({ type: 'error', message: `Грешка: ${error.message}` });
    }
  });

  const handleCreateCompany = (formData) => {
    createCompany({
      variables: {
        input: formData
      }
    });
  };

  const handleUpdateCompany = (formData) => {
    updateCompany({
      variables: {
        id: editCompany.id,
        input: formData
      }
    });
  };

  const handleDeleteCompany = () => {
    deleteCompanyMutation({
      variables: {
        id: deleteCompany.id
      }
    });
  };

  const getCompanyUsers = (companyId) => {
    if (!userCompaniesData) return [];
    return userCompaniesData.userCompanies.filter(uc => uc.companyId === companyId && uc.isActive);
  };

  if (companiesLoading || userCompaniesLoading) {
    return <div>Зареждане...</div>;
  }

  if (companiesError) {
    return <Alert severity="error">Грешка при зареждане на данните: {companiesError.message}</Alert>;
  }

  return (
    <Container maxWidth="xl">
      <Box sx={{ mt: 3, mb: 3 }}>
        <Typography variant="h4" component="h1" gutterBottom>
          Управление на фирми
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
            setEditCompany(null);
            setOpenDialog(true);
          }}
          sx={{ mb: 2 }}
        >
          Добави фирма
        </Button>

        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>ID</TableCell>
                <TableCell>Наименование</TableCell>
                <TableCell>ЕИК</TableCell>
                <TableCell>ДДС номер</TableCell>
                <TableCell>Град</TableCell>
                <TableCell>Потребители</TableCell>
                <TableCell>Статус</TableCell>
                <TableCell>Действия</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {companiesData?.companies?.map((company) => {
                const users = getCompanyUsers(company.id);

                return (
                  <TableRow key={company.id}>
                    <TableCell>{company.id}</TableCell>
                    <TableCell>{company.name}</TableCell>
                    <TableCell>{company.eik}</TableCell>
                    <TableCell>{company.vatNumber || 'N/A'}</TableCell>
                    <TableCell>{company.city || 'N/A'}</TableCell>
                    <TableCell>
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                        {users.map((uc) => (
                          <Chip
                            key={uc.id}
                            label={`${uc.user.firstName} ${uc.user.lastName} (${uc.role})`}
                            size="small"
                            icon={<PersonIcon />}
                            variant="outlined"
                          />
                        ))}
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={company.isActive ? 'Активна' : 'Неактивна'}
                        color={company.isActive ? 'success' : 'default'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell>
                      <Button
                        size="small"
                        startIcon={<EditIcon />}
                        onClick={() => {
                          setEditCompany(company);
                          setOpenDialog(true);
                        }}
                      >
                        Редактиране
                      </Button>
                      <Button
                        size="small"
                        color="error"
                        startIcon={<DeleteIcon />}
                        onClick={() => setDeleteCompany(company)}
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

      {/* Create/Edit Company Dialog */}
      <CompanyForm
        open={openDialog}
        handleClose={() => {
          setOpenDialog(false);
          setEditCompany(null);
        }}
        company={editCompany}
        onSubmit={editCompany ? handleUpdateCompany : handleCreateCompany}
        isEdit={!!editCompany}
      />

      {/* Delete Confirmation Dialog */}
      <Dialog open={!!deleteCompany} onClose={() => setDeleteCompany(null)}>
        <DialogTitle>Потвърждение за изтриване</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Сигурни ли сте, че искате да изтриете фирмата "{deleteCompany?.name}"?
            Това действие не може да бъде отменено и ще изтрие всички свързани данни.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteCompany(null)}>Отказ</Button>
          <Button onClick={handleDeleteCompany} color="error" variant="contained">
            Изтриване
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
};

export default AdminCompanies;