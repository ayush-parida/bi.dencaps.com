import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { TokenService } from '../services/token.service';
import { catchError, throwError } from 'rxjs';

export const authInterceptor: HttpInterceptorFn = (req, next) => {
  const tokenService = inject(TokenService);
  const token = tokenService.getAccessToken();

  // Clone request and add authorization header if token exists
  if (token && !req.url.includes('/auth/')) {
    req = req.clone({
      setHeaders: {
        Authorization: `Bearer ${token}`
      }
    });
  }

  return next(req).pipe(
    catchError(error => {
      // Only clear tokens on 401 if this is a request that had a token attached
      // This indicates the token is invalid/expired
      if (error.status === 401 && token && !req.url.includes('/auth/')) {
        console.warn('Token expired or invalid, clearing tokens');
        tokenService.clearTokens();
        // Don't redirect here - let the auth guard handle navigation
        // This allows components to handle the error appropriately
      }

      return throwError(() => error);
    })
  );
};
