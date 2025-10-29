'use client'

import type { FunctionComponent } from 'react'
import Link from 'next/link'
import AppBar from '@mui/material/AppBar'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import Stack from '@mui/material/Stack'
import Toolbar from '@mui/material/Toolbar'
import Typography from '@mui/material/Typography'
import LibraryAddOutlinedIcon from '@mui/icons-material/LibraryAddOutlined'

import SearchBar from '@/components/SearchBar'

import styles from './styles.module.scss'

const Header: FunctionComponent = () => (
  <AppBar>
    <Toolbar className={styles.toolbar}>
      <Stack className={styles.inner} alignItems="center" direction="row">
        <Typography variant="h6" noWrap component="div">
          Hoarder
        </Typography>
        <Stack className={styles.nav} spacing={1} direction="row">
          <Button href="/" LinkComponent={Link} color="inherit">
            ホーム
          </Button>
          <Button href="/tags" LinkComponent={Link} color="inherit">
            タグ
          </Button>
          <Button href="/sources" LinkComponent={Link} color="inherit">
            サービス
          </Button>
        </Stack>
      </Stack>
      <SearchBar className={styles.inner} />
      <Stack className={styles.inner} justifyContent="end" direction="row">
        <IconButton href="/media/new" LinkComponent={Link} size="large" color="inherit">
          <LibraryAddOutlinedIcon />
        </IconButton>
      </Stack>
    </Toolbar>
  </AppBar>
)

export default Header
