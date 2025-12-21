import React, { useState } from 'react';
import { useQuery } from '@apollo/client';
import { gql } from '@apollo/client';
import { 
  Card, 
  CardContent, 
  CardHeader, 
  CardTitle,
  Button,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Badge,
  Input,
  toast
} from '../ui';
import { Download, Filter, Calendar, TrendingUp, TrendingDown } from 'lucide-react';

const GET_INTRASTAT_REPORT_DATA = gql`
  query GetIntrastatReportData($companyId: Int!, $year: Int!, $month: Int!, $declarationType: DeclarationType) {
    intrastatDeclarations(companyId: $companyId, year: $year, month: $month) {
      id
      declarationType
      totalItems
      totalInvoiceValue
      totalStatisticalValue
      status
    }
    intrastatDeclarationItems(declarationId: $declarationId) {
      id
      cnCode
      countryOfOrigin
      countryOfConsignment
      netMassKg
      invoiceValue
      statisticalValue
      description
      transactionNatureCode
      journalEntryId
      entryLineId
    }
  }
`;

const GET_THRESHOLD_ANALYSIS = gql`
  query GetThresholdAnalysis($companyId: Int!, $year: Int!) {
    intrastatThresholdAnalysis(companyId: $companyId, year: $year) {
      month
      arrivalValue
      dispatchValue
      arrivalCumulative
      dispatchCumulative
      arrivalThresholdExceeded
      dispatchThresholdExceeded
    }
  }
`;

const IntrastatReports = ({ companyId }) => {
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear());
  const [selectedMonth, setSelectedMonth] = useState(new Date().getMonth() + 1);
  const [reportType, setReportType] = useState('summary');
  const [filterCountry, setFilterCountry] = useState('');
  const [filterCnCode, setFilterCnCode] = useState('');
  
  const currentYear = new Date().getFullYear();
  const years = Array.from({ length: 5 }, (_, i) => currentYear - i);
  const months = [
    { value: 1, label: 'Януари' },
    { value: 2, label: 'Февруари' },
    { value: 3, label: 'Март' },
    { value: 4, label: 'Април' },
    { value: 5, label: 'Май' },
    { value: 6, label: 'Юни' },
    { value: 7, label: 'Юли' },
    { value: 8, label: 'Август' },
    { value: 9, label: 'Септември' },
    { value: 10, label: 'Октомври' },
    { value: 11, label: 'Ноември' },
    { value: 12, label: 'Декември' }
  ];

  const { data: reportData, loading: reportLoading } = useQuery(GET_INTRASTAT_REPORT_DATA, {
    variables: { companyId, year: selectedYear, month: selectedMonth }
  });

  const { data: thresholdData, loading: thresholdLoading } = useQuery(GET_THRESHOLD_ANALYSIS, {
    variables: { companyId, year: selectedYear }
  });

  const thresholdRows = thresholdData?.intrastatThresholdAnalysis || [];

  const exportToExcel = (data, filename) => {
    const csvContent = "data:text/csv;charset=utf-8," 
      + Object.keys(data[0]).join(",") + "\n"
      + data.map(row => Object.values(row).join(",")).join("\n");
    
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `${filename}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    toast({
      title: "Успех",
      description: "Справката е експортирана успешно"
    });
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('bg-BG', {
      style: 'currency',
      currency: 'BGN',
      minimumFractionDigits: 2
    }).format(amount);
  };

  if (reportLoading || thresholdLoading) {
    return <div className="flex justify-center items-center h-64">Зареждане...</div>;
  }

  const declarations = reportData?.intrastatDeclarations || [];
  const arrivalDeclarations = declarations.filter(d => d.declarationType === 'ARRIVAL');
  const dispatchDeclarations = declarations.filter(d => d.declarationType === 'DISPATCH');
  
  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">INTRASTAT Справки</h1>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => window.print()}>
            <Calendar className="w-4 h-4 mr-2" />
            Печат
          </Button>
        </div>
      </div>

      {/* Филтри */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Filter className="w-5 h-5" />
            Филтри
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="text-sm font-medium">Година</label>
              <Select value={selectedYear.toString()} onValueChange={(v) => setSelectedYear(Number(v))}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {years.map(year => (
                    <SelectItem key={year} value={year.toString()}>{year}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <label className="text-sm font-medium">Месец</label>
              <Select value={selectedMonth.toString()} onValueChange={(v) => setSelectedMonth(Number(v))}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {months.map(month => (
                    <SelectItem key={month.value} value={month.value.toString()}>
                      {month.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <label className="text-sm font-medium">Страна</label>
              <Input 
                placeholder="напр. DE, FR, IT"
                value={filterCountry}
                onChange={(e) => setFilterCountry(e.target.value)}
              />
            </div>
            <div>
              <label className="text-sm font-medium">CN код</label>
              <Input 
                placeholder="напр. 84832000"
                value={filterCnCode}
                onChange={(e) => setFilterCnCode(e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <Tabs value={reportType} onValueChange={setReportType}>
        <TabsList>
          <TabsTrigger value="summary">Обобщение</TabsTrigger>
          <TabsTrigger value="arrivals">Входящи</TabsTrigger>
          <TabsTrigger value="dispatches">Изходящи</TabsTrigger>
          <TabsTrigger value="threshold">Анализ на прагове</TabsTrigger>
        </TabsList>

        <TabsContent value="summary">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingDown className="w-5 h-5 text-green-600" />
                  Входящи операции
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {arrivalDeclarations.map(declaration => (
                    <div key={declaration.id} className="p-4 border rounded-lg">
                      <div className="flex justify-between items-center mb-2">
                        <Badge variant="outline">Входящи</Badge>
                        <Badge variant={declaration.status === 'SUBMITTED' ? 'default' : 'secondary'}>
                          {declaration.status === 'DRAFT' ? 'Чернова' : 
                           declaration.status === 'SUBMITTED' ? 'Подадена' : 
                           declaration.status === 'ACCEPTED' ? 'Приета' : 'Отхвърлена'}
                        </Badge>
                      </div>
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <span className="text-muted-foreground">Артикули:</span>
                          <span className="ml-2 font-semibold">{declaration.totalItems}</span>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Стойност:</span>
                          <span className="ml-2 font-semibold">
                            {formatCurrency(declaration.totalInvoiceValue)}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                  {arrivalDeclarations.length === 0 && (
                    <p className="text-center text-muted-foreground py-8">
                      Няма входящи операции за избрания период
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="w-5 h-5 text-blue-600" />
                  Изходящи операции
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {dispatchDeclarations.map(declaration => (
                    <div key={declaration.id} className="p-4 border rounded-lg">
                      <div className="flex justify-between items-center mb-2">
                        <Badge variant="outline">Изходящи</Badge>
                        <Badge variant={declaration.status === 'SUBMITTED' ? 'default' : 'secondary'}>
                          {declaration.status === 'DRAFT' ? 'Чернова' : 
                           declaration.status === 'SUBMITTED' ? 'Подадена' : 
                           declaration.status === 'ACCEPTED' ? 'Приета' : 'Отхвърлена'}
                        </Badge>
                      </div>
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <span className="text-muted-foreground">Артикули:</span>
                          <span className="ml-2 font-semibold">{declaration.totalItems}</span>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Стойност:</span>
                          <span className="ml-2 font-semibold">
                            {formatCurrency(declaration.totalInvoiceValue)}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                  {dispatchDeclarations.length === 0 && (
                    <p className="text-center text-muted-foreground py-8">
                      Няма изходящи операции за избрания период
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="arrivals">
          <Card>
            <CardHeader>
              <CardTitle className="flex justify-between items-center">
                <span className="flex items-center gap-2">
                  <TrendingDown className="w-5 h-5 text-green-600" />
                  Детайли за входящи операции
                </span>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => exportToExcel(arrivalDeclarations, `intrastat_arrivals_${selectedYear}_${selectedMonth}`)}
                >
                  <Download className="w-4 h-4 mr-2" />
                  Експорт
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>CN Код</TableHead>
                    <TableHead>Описание</TableHead>
                    <TableHead>Страна произход</TableHead>
                    <TableHead>Тегло (кг)</TableHead>
                    <TableHead>Стойност</TableHead>
                    <TableHead>Статус</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {arrivalDeclarations.map(declaration => 
                    declaration.items?.map(item => (
                      <TableRow key={item.id}>
                        <TableCell className="font-mono">{item.cnCode}</TableCell>
                        <TableCell>{item.description}</TableCell>
                        <TableCell>{item.countryOfOrigin}</TableCell>
                        <TableCell className="text-right">{Number(item.netMassKg).toLocaleString()}</TableCell>
                        <TableCell className="text-right">{formatCurrency(item.invoiceValue)}</TableCell>
                        <TableCell>
                          <Badge variant="outline">Входящи</Badge>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="dispatches">
          <Card>
            <CardHeader>
              <CardTitle className="flex justify-between items-center">
                <span className="flex items-center gap-2">
                  <TrendingUp className="w-5 h-5 text-blue-600" />
                  Детайли за изходящи операции
                </span>
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => exportToExcel(dispatchDeclarations, `intrastat_dispatches_${selectedYear}_${selectedMonth}`)}
                >
                  <Download className="w-4 h-4 mr-2" />
                  Експорт
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>CN Код</TableHead>
                    <TableHead>Описание</TableHead>
                    <TableHead>Страна назначение</TableHead>
                    <TableHead>Тегло (кг)</TableHead>
                    <TableHead>Стойност</TableHead>
                    <TableHead>Статус</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {dispatchDeclarations.map(declaration => 
                    declaration.items?.map(item => (
                      <TableRow key={item.id}>
                        <TableCell className="font-mono">{item.cnCode}</TableCell>
                        <TableCell>{item.description}</TableCell>
                        <TableCell>{item.countryOfConsignment}</TableCell>
                        <TableCell className="text-right">{Number(item.netMassKg).toLocaleString()}</TableCell>
                        <TableCell className="text-right">{formatCurrency(item.invoiceValue)}</TableCell>
                        <TableCell>
                          <Badge variant="outline">Изходящи</Badge>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="threshold">
          <Card>
            <CardHeader>
              <CardTitle>Анализ на прагове за {selectedYear} година</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                  <div className="p-4 bg-green-50 rounded-lg">
                    <h3 className="font-semibold text-green-800 mb-2">Входящи операции</h3>
                    <p className="text-2xl font-bold text-green-600">
                      {formatCurrency(arrivalDeclarations.reduce((sum, d) => sum + Number(d.totalInvoiceValue), 0))}
                    </p>
                    <p className="text-sm text-green-600">за избрания месец</p>
                  </div>
                  <div className="p-4 bg-blue-50 rounded-lg">
                    <h3 className="font-semibold text-blue-800 mb-2">Изходящи операции</h3>
                    <p className="text-2xl font-bold text-blue-600">
                      {formatCurrency(dispatchDeclarations.reduce((sum, d) => sum + Number(d.totalInvoiceValue), 0))}
                    </p>
                    <p className="text-sm text-blue-600">за избрания месец</p>
                  </div>
                </div>

                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Месец</TableHead>
                      <TableHead>Входящи</TableHead>
                      <TableHead>Изходящи</TableHead>
                      <TableHead>Кумулативно входящи</TableHead>
                      <TableHead>Кумулативно изходящи</TableHead>
                      <TableHead>Статус</TableHead>
                    </TableRow>
                  </TableHeader>
                <TableBody>
                  {thresholdRows.length > 0 ? (
                    thresholdRows.map((row) => {
                      const arrivalExceeded = row.arrivalThresholdExceeded;
                      const dispatchExceeded = row.dispatchThresholdExceeded;

                      return (
                        <TableRow key={row.month}>
                          <TableCell>{months[row.month - 1]?.label || row.month}</TableCell>
                          <TableCell>{formatCurrency(row.arrivalValue)}</TableCell>
                          <TableCell>{formatCurrency(row.dispatchValue)}</TableCell>
                          <TableCell className={arrivalExceeded ? 'text-red-600 font-semibold' : ''}>
                            {formatCurrency(row.arrivalCumulative)}
                          </TableCell>
                          <TableCell className={dispatchExceeded ? 'text-red-600 font-semibold' : ''}>
                            {formatCurrency(row.dispatchCumulative)}
                          </TableCell>
                          <TableCell>
                            {arrivalExceeded || dispatchExceeded ? (
                              <Badge variant="destructive">Прагът е надвишен</Badge>
                            ) : (
                              <Badge variant="secondary">Под прага</Badge>
                            )}
                          </TableCell>
                        </TableRow>
                      );
                    })
                  ) : (
                    months.slice(0, selectedMonth).map((month) => (
                      <TableRow key={month.value}>
                        <TableCell>{month.label}</TableCell>
                        <TableCell>{formatCurrency(0)}</TableCell>
                        <TableCell>{formatCurrency(0)}</TableCell>
                        <TableCell>{formatCurrency(0)}</TableCell>
                        <TableCell>{formatCurrency(0)}</TableCell>
                        <TableCell>
                          <Badge variant="secondary">Няма данни</Badge>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default IntrastatReports;
